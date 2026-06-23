// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! One-shot PipeWire connection for non-interactive CLI commands.
//!
//! Spawns a [`Session`], pumps events until the initial enumeration completes
//! ([`Event::Ready`]), and snapshots the [`State`]. Unlike the TUI, errors are
//! surfaced rather than ignored (Standard §3.1).

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::cli::error::{AppError, ErrorCode};
use crate::wirehose::state::State;
use crate::wirehose::{Event, Session};

/// How long to wait for PipeWire's initial state enumeration. Generous enough
/// for a busy graph on a loaded machine, short enough to fail fast when no
/// PipeWire daemon is reachable.
const READY_TIMEOUT: Duration = Duration::from_secs(5);

/// How long to wait for a write command's effect to be reflected back as a
/// state event before reporting it as merely submitted.
const CONFIRM_TIMEOUT: Duration = Duration::from_millis(1500);

/// A live one-shot connection: a snapshot of PipeWire state plus the channel
/// and session kept alive so writes can be confirmed.
pub struct Connection {
    rx: Receiver<Event>,
    state: State,
    // Dropped last: `Session::drop` signals the monitor thread and joins it.
    session: Session,
}

impl Connection {
    /// Connect, wait for the initial snapshot, and return it.
    pub fn open(remote: Option<String>) -> Result<Self, AppError> {
        let (tx, rx) = mpsc::channel::<Event>();
        let handler = move |event| tx.send(event).is_ok();
        let session = Session::spawn(remote, handler).map_err(|e| {
            AppError::unavailable(format!("failed to connect to PipeWire: {e}"))
                .with_hint(
                    "check PipeWire is running (`systemctl --user status \
                     pipewire`) or pass --remote <name>",
                )
        })?;

        let mut state = State::default();
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(remaining) {
                Ok(Event::Ready) => break,
                Ok(Event::State(event)) => {
                    state.update(event);
                }
                Ok(Event::Error(message)) => {
                    return Err(AppError::unavailable(format!(
                        "PipeWire error: {message}"
                    )));
                }
                Err(RecvTimeoutError::Timeout) => {
                    return Err(AppError::new(
                        ErrorCode::Timeout,
                        "timed out waiting for PipeWire state",
                    )
                    .with_hint(
                        "check PipeWire is running or pass --remote <name>",
                    ));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(AppError::unavailable(
                        "PipeWire monitor thread stopped unexpectedly",
                    ));
                }
            }
        }

        // Drain anything already queued so the snapshot is as fresh as possible.
        while let Ok(Event::State(event)) = rx.try_recv() {
            state.update(event);
        }

        Ok(Self { rx, state, session })
    }

    /// The snapshotted PipeWire state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// The command sink, for issuing writes (`CommandSender`).
    pub fn sender(&self) -> &dyn crate::wirehose::CommandSender {
        &self.session
    }

    /// After issuing a write, pump the resulting events into the snapshot: wait
    /// briefly for the first effect, then drain until a short idle gap (or the
    /// hard deadline). A subsequent re-read then reflects the change.
    pub fn settle(&mut self) {
        let hard = Instant::now() + CONFIRM_TIMEOUT;
        // Grant the first resulting event more grace than later ones.
        let mut idle = Duration::from_millis(400);
        loop {
            let now = Instant::now();
            if now >= hard {
                break;
            }
            let wait = idle.min(hard.saturating_duration_since(now));
            match self.rx.recv_timeout(wait) {
                Ok(Event::State(event)) => {
                    self.state.update(event);
                    idle = Duration::from_millis(120);
                }
                Ok(_) => {}
                // Idle gap or disconnect: the change has settled.
                Err(_) => break,
            }
        }
    }
}
