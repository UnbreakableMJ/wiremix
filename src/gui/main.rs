// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `wiremix-gui` — a Slint desktop frontend for the PipeWire mixer.
//!
//! A third frontend alongside the TUI and CLI, driving the same UI-agnostic
//! [`wiremix::wirehose`] session and [`wiremix::view::View`] projection in
//! process. The PipeWire monitor runs on its own thread (as in the TUI);
//! events arrive over an `mpsc` channel that a Slint [`Timer`] drains on the
//! UI thread. Structural changes rebuild the `View` and refill the Slint
//! models; high-frequency peak meters update a separate model so they never
//! disturb the volume sliders. Every control routes back through the `Logic`
//! global into `View` command methods against the live session.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{mpsc, Arc};
use std::time::Duration;

use anyhow::Result;
use slint::{
    ComponentHandle, Model, ModelRc, SharedString, Timer, TimerMode, VecModel,
};

use wiremix::config::{Config, Peaks, TabKind};
use wiremix::device_kind::DeviceKind;
use wiremix::opt::Opt;
use wiremix::view::{ListKind, NodeKind, View, VolumeAdjustment};
use wiremix::wirehose::media_class;
use wiremix::wirehose::state::{CaptureEligibility, State};
use wiremix::wirehose::{
    CommandSender, Event as PwEvent, ObjectId, PeakProcessor, Session,
    StateEvent,
};

slint::include_modules!();

/// VU-meter refresh cadence (~30 fps), matching the TUI's default feel.
const TICK: Duration = Duration::from_millis(33);

/// All UI-thread-owned mutable state. Shared via `Rc<RefCell<_>>` between the
/// event-pump timer and every `Logic` callback (single-threaded, so borrows
/// never overlap).
struct Gui {
    state: State,
    config: Config,
    current_tab: usize,
    ready: bool,
    /// Object IDs of the rows currently shown, parallel to the node/device
    /// models, so an index from a Slint callback maps back to an `ObjectId`.
    node_ids: Vec<ObjectId>,
    device_ids: Vec<ObjectId>,
    /// Per node row: the shared peak buffer and whether it is stereo. `None`
    /// for rows without a capture buffer (meter stays idle).
    peak_sources: Vec<Option<(Arc<[wiremix::atomic_f32::AtomicF32]>, bool)>>,
    /// `Some` on the Output/Input tabs, naming which default this tab sets.
    default_kind: Option<DeviceKind>,
    /// Nodes with an active capture stream.
    capturing: HashSet<ObjectId>,
    peak_processor: Arc<dyn PeakProcessor>,
}

/// The owned result of projecting `State` for the current tab — applied to the
/// Slint models and cached back into `Gui` by [`apply`].
struct Projection {
    node_rows: Vec<NodeRow>,
    peak_rows: Vec<PeakRow>,
    peak_sources: Vec<Option<(Arc<[wiremix::atomic_f32::AtomicF32]>, bool)>>,
    device_rows: Vec<DeviceRow>,
    node_ids: Vec<ObjectId>,
    device_ids: Vec<ObjectId>,
    is_device_tab: bool,
    default_kind: Option<DeviceKind>,
}

/// Map a tab to its list projection and (for device tabs) the default it sets.
fn tab_info(tab: TabKind) -> (ListKind, Option<DeviceKind>) {
    match tab {
        TabKind::Playback => (ListKind::Node(NodeKind::Playback), None),
        TabKind::Recording => (ListKind::Node(NodeKind::Recording), None),
        TabKind::Output => {
            (ListKind::Node(NodeKind::Output), Some(DeviceKind::Sink))
        }
        TabKind::Input => {
            (ListKind::Node(NodeKind::Input), Some(DeviceKind::Source))
        }
        TabKind::Configuration => (ListKind::Device, None),
    }
}

fn tab_title(tab: TabKind) -> &'static str {
    match tab {
        TabKind::Playback => "Playback",
        TabKind::Recording => "Recording",
        TabKind::Output => "Output Devices",
        TabKind::Input => "Input Devices",
        TabKind::Configuration => "Configuration",
    }
}

fn str_model(items: Vec<SharedString>) -> ModelRc<SharedString> {
    Rc::new(VecModel::from(items)).into()
}

/// Rebuild the `View` and project the current tab into owned model rows. Reads
/// `Gui` immutably (the borrowed `View` is dropped before the result returns).
fn project(gui: &Gui, session: &Session) -> Projection {
    let view =
        View::from(session, &gui.state, &gui.config.names, &gui.config.filters);

    let tab = gui
        .config
        .tabs
        .get(gui.current_tab)
        .copied()
        .unwrap_or(TabKind::Playback);
    let (list_kind, default_kind) = tab_info(tab);
    let is_device_tab = matches!(list_kind, ListKind::Device);
    let ids: Vec<ObjectId> = view.object_ids(list_kind).to_vec();

    let mut node_rows = Vec::new();
    let mut peak_rows = Vec::new();
    let mut peak_sources = Vec::new();
    let mut node_ids = Vec::new();
    let mut device_rows = Vec::new();
    let mut device_ids = Vec::new();

    if is_device_tab {
        for id in ids {
            let Some(device) = view.devices.get(&id) else {
                continue;
            };
            let (labels, index) = match view.device_targets(id) {
                Some((targets, selected)) => (
                    targets
                        .iter()
                        .map(|(_, label)| SharedString::from(label.as_str()))
                        .collect(),
                    selected as i32,
                ),
                None => (Vec::new(), 0),
            };
            device_rows.push(DeviceRow {
                title: device.title.as_str().into(),
                profile_labels: str_model(labels),
                profile_index: index,
            });
            device_ids.push(id);
        }
    } else {
        for id in ids {
            let Some(node) = view.nodes.get(&id) else {
                continue;
            };

            let percent = if node.volumes.is_empty() {
                0.0
            } else {
                let avg = node.volumes.iter().sum::<f32>()
                    / node.volumes.len() as f32;
                avg.cbrt() * 100.0
            };

            let (labels, target_index, show_target) = match view
                .node_targets(id)
            {
                Some((targets, selected)) if !targets.is_empty() => (
                    targets
                        .iter()
                        .map(|(_, label)| SharedString::from(label.as_str()))
                        .collect(),
                    selected as i32,
                    true,
                ),
                _ => (Vec::new(), 0, false),
            };

            node_rows.push(NodeRow {
                title: node.title.as_str().into(),
                subtitle: node.target_title.as_str().into(),
                volume: percent,
                muted: node.mute,
                is_default: node.is_default_sink || node.is_default_source,
                can_default: default_kind.is_some(),
                target_labels: str_model(labels),
                target_index,
                show_target,
            });

            let (source, peak_row) = match &node.peaks {
                Some(peaks) if !peaks.is_empty() => {
                    let stereo = peaks.len() >= 2;
                    (
                        Some((Arc::clone(peaks), stereo)),
                        PeakRow {
                            left: 0.0,
                            right: 0.0,
                            stereo,
                            active: true,
                        },
                    )
                }
                _ => (
                    None,
                    PeakRow {
                        left: 0.0,
                        right: 0.0,
                        stereo: false,
                        active: false,
                    },
                ),
            };
            peak_sources.push(source);
            peak_rows.push(peak_row);
            node_ids.push(id);
        }
    }

    Projection {
        node_rows,
        peak_rows,
        peak_sources,
        device_rows,
        node_ids,
        device_ids,
        is_device_tab,
        default_kind,
    }
}

/// Project the current tab and push it into the models and window properties.
fn apply(
    gui: &Rc<RefCell<Gui>>,
    session: &Session,
    nodes_model: &Rc<VecModel<NodeRow>>,
    peaks_model: &Rc<VecModel<PeakRow>>,
    devices_model: &Rc<VecModel<DeviceRow>>,
    win: &MainWindow,
) {
    let projection = {
        let gui = gui.borrow();
        project(&gui, session)
    };

    nodes_model.set_vec(projection.node_rows);
    peaks_model.set_vec(projection.peak_rows);
    devices_model.set_vec(projection.device_rows);
    win.set_is_device_tab(projection.is_device_tab);

    let mut gui = gui.borrow_mut();
    gui.node_ids = projection.node_ids;
    gui.device_ids = projection.device_ids;
    gui.peak_sources = projection.peak_sources;
    gui.default_kind = projection.default_kind;
    win.set_current_tab(gui.current_tab as i32);
    win.set_ready(gui.ready);
}

/// Read the shared peak buffers and refresh the meter model in place.
fn update_peaks(gui: &Rc<RefCell<Gui>>, peaks_model: &Rc<VecModel<PeakRow>>) {
    let gui = gui.borrow();
    let count = peaks_model.row_count();
    for (i, source) in gui.peak_sources.iter().enumerate() {
        if i >= count {
            break;
        }
        let row = match source {
            Some((peaks, stereo)) => {
                let left = peaks.first().map(|p| p.load()).unwrap_or(0.0);
                let right = if *stereo {
                    peaks.get(1).map(|p| p.load()).unwrap_or(left)
                } else {
                    left
                };
                PeakRow {
                    left,
                    right,
                    stereo: *stereo,
                    active: true,
                }
            }
            None => PeakRow {
                left: 0.0,
                right: 0.0,
                stereo: false,
                active: false,
            },
        };
        peaks_model.set_row_data(i, row);
    }
}

/// Start (or, when `force`, restart) a capture stream for live peak metering,
/// mirroring the TUI's eligibility handling.
fn start_capture(gui: &mut Gui, session: &Session, id: ObjectId, force: bool) {
    if matches!(gui.config.peaks, Peaks::Off) {
        return;
    }
    if !force && gui.capturing.contains(&id) {
        return;
    }

    let Some((serial, capture_sink, peaks_dirty)) =
        gui.state.nodes.get(&id).and_then(|node| {
            let object_serial = node.props.object_serial()?;
            let capture_sink =
                node.props.media_class().as_ref().is_some_and(|class| {
                    media_class::is_sink(class) || media_class::is_source(class)
                });
            Some((*object_serial, capture_sink, Arc::clone(&node.peaks_dirty)))
        })
    else {
        return;
    };

    gui.capturing.insert(id);
    let processor = Arc::clone(&gui.peak_processor);
    session.node_capture_start(
        id,
        serial,
        capture_sink,
        peaks_dirty,
        Some(processor),
    );
}

fn handle_eligibility(
    gui: &mut Gui,
    session: &Session,
    eligibility: CaptureEligibility,
) {
    match eligibility {
        CaptureEligibility::Eligible(id) => {
            start_capture(gui, session, id, false)
        }
        CaptureEligibility::NeedsRestart(id) => {
            start_capture(gui, session, id, true)
        }
        CaptureEligibility::Ineligible(id) => {
            if gui.capturing.remove(&id) {
                session.node_capture_stop(id);
            }
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let config_default_path = Config::default_path();
    let config_path = opt.config.as_deref().or(config_default_path.as_deref());
    let config = Config::try_new(config_path, &opt)?;

    // Read what we need before moving `config` into the shared state.
    let remote = config.remote.clone();
    let tab_titles: Vec<SharedString> =
        config.tabs.iter().map(|t| tab_title(*t).into()).collect();
    let initial_tab = config.tab.min(config.tabs.len().saturating_sub(1));

    // PipeWire events flow UI-ward over this channel.
    let (event_tx, event_rx) = mpsc::channel::<PwEvent>();
    let session = Rc::new(Session::spawn(remote, move |event| {
        event_tx.send(event).is_ok()
    })?);

    // VU-meter ballistics, identical to the TUI (300 ms attack/release).
    let peak_processor: Arc<dyn PeakProcessor> = Arc::new(
        |new_peak: f32, current_peak: f32, samples: u32, rate: u32| {
            let time_constant = 0.3;
            let coef =
                1.0 - (-(samples as f32) / (time_constant * rate as f32)).exp();
            current_peak + (new_peak - current_peak) * coef
        },
    );

    let gui = Rc::new(RefCell::new(Gui {
        state: State::default(),
        config,
        current_tab: initial_tab,
        ready: false,
        node_ids: Vec::new(),
        device_ids: Vec::new(),
        peak_sources: Vec::new(),
        default_kind: None,
        capturing: HashSet::new(),
        peak_processor,
    }));

    let window =
        MainWindow::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let nodes_model = Rc::new(VecModel::<NodeRow>::default());
    let peaks_model = Rc::new(VecModel::<PeakRow>::default());
    let devices_model = Rc::new(VecModel::<DeviceRow>::default());

    window.set_tab_titles(str_model(tab_titles));
    window.set_current_tab(initial_tab as i32);
    window.set_nodes(nodes_model.clone().into());
    window.set_peaks(peaks_model.clone().into());
    window.set_devices(devices_model.clone().into());

    // Wire the controls. Each closure rebuilds a transient `View` borrowing the
    // session and runs the matching command; the resulting state change comes
    // back as events and re-projects on the next tick.
    let logic = window.global::<Logic>();

    {
        let gui = gui.clone();
        let session = session.clone();
        logic.on_set_volume(move |index, percent| {
            let gui = gui.borrow();
            let Some(&id) = gui.node_ids.get(index as usize) else {
                return;
            };
            let view = View::from(
                &*session,
                &gui.state,
                &gui.config.names,
                &gui.config.filters,
            );
            view.volume(id, VolumeAdjustment::Absolute(percent / 100.0), None);
        });
    }

    {
        let gui = gui.clone();
        let session = session.clone();
        logic.on_toggle_mute(move |index| {
            let gui = gui.borrow();
            let Some(&id) = gui.node_ids.get(index as usize) else {
                return;
            };
            let view = View::from(
                &*session,
                &gui.state,
                &gui.config.names,
                &gui.config.filters,
            );
            view.mute(id);
        });
    }

    {
        let gui = gui.clone();
        let session = session.clone();
        logic.on_set_default(move |index| {
            let gui = gui.borrow();
            let Some(&id) = gui.node_ids.get(index as usize) else {
                return;
            };
            let Some(kind) = gui.default_kind else {
                return;
            };
            let view = View::from(
                &*session,
                &gui.state,
                &gui.config.names,
                &gui.config.filters,
            );
            view.set_default(id, kind);
        });
    }

    {
        let gui = gui.clone();
        let session = session.clone();
        logic.on_set_target(move |index, option| {
            let gui = gui.borrow();
            let Some(&id) = gui.node_ids.get(index as usize) else {
                return;
            };
            let view = View::from(
                &*session,
                &gui.state,
                &gui.config.names,
                &gui.config.filters,
            );
            if let Some((targets, _)) = view.node_targets(id) {
                if let Some(target) =
                    targets.get(option as usize).map(|(t, _)| *t)
                {
                    view.set_target(id, target);
                }
            }
        });
    }

    {
        let gui = gui.clone();
        let session = session.clone();
        logic.on_set_profile(move |index, option| {
            let gui = gui.borrow();
            let Some(&id) = gui.device_ids.get(index as usize) else {
                return;
            };
            let view = View::from(
                &*session,
                &gui.state,
                &gui.config.names,
                &gui.config.filters,
            );
            if let Some((targets, _)) = view.device_targets(id) {
                if let Some(target) =
                    targets.get(option as usize).map(|(t, _)| *t)
                {
                    view.set_target(id, target);
                }
            }
        });
    }

    {
        let gui = gui.clone();
        let session = session.clone();
        let nodes_model = nodes_model.clone();
        let peaks_model = peaks_model.clone();
        let devices_model = devices_model.clone();
        let window_weak = window.as_weak();
        logic.on_select_tab(move |index| {
            {
                let mut gui = gui.borrow_mut();
                gui.current_tab = index as usize;
            }
            if let Some(window) = window_weak.upgrade() {
                apply(
                    &gui,
                    &session,
                    &nodes_model,
                    &peaks_model,
                    &devices_model,
                    &window,
                );
            }
        });
    }

    // The event pump: drain PipeWire events, re-project on structural change,
    // and refresh meters every tick.
    let timer = Timer::default();
    {
        let gui = gui.clone();
        let session = session.clone();
        let nodes_model = nodes_model.clone();
        let peaks_model = peaks_model.clone();
        let devices_model = devices_model.clone();
        let window_weak = window.as_weak();
        timer.start(TimerMode::Repeated, TICK, move || {
            let mut dirty = false;
            let mut became_ready = false;

            while let Ok(event) = event_rx.try_recv() {
                match event {
                    PwEvent::Ready => became_ready = true,
                    PwEvent::Error(_) => {}
                    PwEvent::State(state_event) => {
                        let structural = !matches!(
                            &state_event,
                            StateEvent::NodePeaksDirty { .. }
                        );
                        let mut gui = gui.borrow_mut();
                        let eligibilities = gui.state.update(state_event);
                        for eligibility in eligibilities {
                            handle_eligibility(&mut gui, &session, eligibility);
                        }
                        dirty |= structural;
                    }
                }
            }

            if became_ready {
                gui.borrow_mut().ready = true;
                dirty = true;
            }

            let Some(window) = window_weak.upgrade() else {
                return;
            };
            if dirty {
                apply(
                    &gui,
                    &session,
                    &nodes_model,
                    &peaks_model,
                    &devices_model,
                    &window,
                );
            }
            update_peaks(&gui, &peaks_model);
        });
    }

    window.run().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(())
}
