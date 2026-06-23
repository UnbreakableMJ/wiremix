// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The machine-readable noun-verb CLI (Spacecraft Software CLI Standard, SFRS).
//!
//! This runs *alongside* the interactive TUI. `main` dispatches a subcommand
//! here as a one-shot command; with no subcommand it runs the TUI. Everything
//! is built on the existing [`wirehose`](crate::wirehose) layer and the
//! [`View`], so CLI and TUI agree on volume, routing, mute,
//! and defaults.

pub mod dto;
pub mod envelope;
pub mod error;
pub mod oneshot;
pub mod output;
pub mod schema;

use std::io::Write;

use serde::Serialize;
use serde_json::{json, Value};

use crate::config::Config;
use crate::device_kind::DeviceKind;
use crate::view::{View, VolumeAdjustment};
use crate::wirehose::{media_class, ObjectId};

use error::{AppError, ErrorCode, ExitCode};
use oneshot::Connection;
use output::{Mode, Resolved};

/// Tool identity and attribution (Standard §15).
pub const TOOL: &str = "wiremix";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAINTAINER: &str =
    "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>";
pub const WEBSITE: &str = "https://Wiremix.SpacecraftSoftware.org/";
pub const DOCS_URL: &str = "https://Wiremix.SpacecraftSoftware.org/";

/// Global flags shared by every subcommand (CLI Standard §3). Flattened into
/// the top-level parser as `global = true` so they may appear before or after
/// the subcommand.
#[derive(clap::Args, Debug, Default, Clone)]
pub struct GlobalArgs {
    /// Machine-readable JSON output (alias for --format json).
    #[clap(long, global = true)]
    pub json: bool,

    /// Output format (explore = interactive TUI).
    #[clap(long, global = true, value_enum, value_name = "FORMAT")]
    pub format: Option<output::Format>,

    /// Restrict output records to these fields (comma-separated).
    #[clap(long, global = true, value_delimiter = ',', value_name = "FIELDS")]
    pub fields: Option<Vec<String>>,

    /// Emit the action plan as JSON and make no changes.
    #[clap(long, global = true)]
    pub dry_run: bool,

    /// Diagnostic output to stderr.
    #[clap(long, global = true)]
    pub verbose: bool,

    /// Suppress non-error stderr.
    #[clap(short = 'q', long, global = true)]
    pub quiet: bool,

    /// When to use color (output is plain text; accepted for compatibility).
    #[clap(long, global = true, value_enum, value_name = "WHEN")]
    pub color: Option<output::ColorWhen>,

    /// Disable color.
    #[clap(long, global = true)]
    pub no_color: bool,

    /// Render absolute time (output is always ISO 8601 UTC regardless).
    #[clap(long, global = true)]
    pub absolute_time: bool,

    /// NUL-terminate output for filename-safe piping.
    #[clap(short = '0', long, global = true)]
    pub print0: bool,

    /// Skip confirmation in non-TTY mode.
    #[clap(long, global = true)]
    pub yes: bool,

    /// Alias for --yes.
    #[clap(long, global = true)]
    pub force: bool,
}

/// The noun-verb command tree.
#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Inspect and control nodes (streams and device endpoints).
    #[clap(subcommand)]
    Node(NodeCmd),
    /// Inspect and configure devices (profiles).
    #[clap(subcommand)]
    Device(DeviceCmd),
    /// Inspect links between nodes.
    #[clap(subcommand)]
    Link(LinkCmd),
    /// Inspect PipeWire metadata (e.g. defaults).
    #[clap(subcommand)]
    Metadata(MetadataCmd),
    /// Show server/session summary.
    #[clap(subcommand)]
    Server(ServerCmd),
    /// Print the JSON Schema of the CLI.
    Schema,
    /// Print a capability manifest of the CLI.
    Describe,
}

#[derive(clap::Subcommand, Debug)]
pub enum NodeCmd {
    /// List controllable nodes.
    List,
    /// Show one node.
    Get { id: ObjectId },
    /// Set a node's volume (percent, all channels).
    SetVolume { id: ObjectId, percent: f32 },
    /// Mute, unmute, or toggle a node (default: toggle).
    Mute {
        id: ObjectId,
        /// Mute the node.
        #[clap(long, conflicts_with_all = ["off", "toggle"])]
        on: bool,
        /// Unmute the node.
        #[clap(long, conflicts_with_all = ["on", "toggle"])]
        off: bool,
        /// Toggle mute (the default).
        #[clap(long, conflicts_with_all = ["on", "off"])]
        toggle: bool,
    },
    /// Set a node as the default sink or source.
    SetDefault { id: ObjectId },
}

#[derive(clap::Subcommand, Debug)]
pub enum DeviceCmd {
    /// List devices.
    List,
    /// Show one device.
    Get { id: ObjectId },
    /// Switch a device to a profile by index.
    SetProfile { id: ObjectId, profile: i32 },
}

#[derive(clap::Subcommand, Debug)]
pub enum LinkCmd {
    /// List links.
    List,
}

#[derive(clap::Subcommand, Debug)]
pub enum MetadataCmd {
    /// List metadata objects.
    List,
    /// Show one metadata object by name (default: "default").
    Get {
        #[clap(long, default_value = "default")]
        name: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ServerCmd {
    /// Show default sink/source, object counts, and remote.
    Info,
}

/// Run a one-shot CLI command and return the process exit code. The PipeWire
/// connection is dropped (cleanly shutting down its thread) before returning.
pub fn run(command: Command, global: &GlobalArgs, config: Config) -> ExitCode {
    let command_str = command_label(&command);

    let resolved = output::resolve(global.json, global.format);
    let mode = match resolved {
        Resolved::Output(mode) => mode,
        // explore + a subcommand is a misuse; report it in a sensible mode.
        Resolved::Tui if global.json => Mode::Json,
        Resolved::Tui => Mode::Human,
    };

    let result = match resolved {
        Resolved::Tui => Err(AppError::usage(
            "`--format explore` is the interactive TUI and cannot be combined \
             with a subcommand",
        )
        .with_hint("wiremix")),
        Resolved::Output(_) => dispatch(command, global, &config, &command_str),
    };

    match result {
        Ok(value) => emit_ok(value, mode, global),
        Err(err) => emit_error(&err, mode, &command_str),
    }
}

fn command_label(command: &Command) -> String {
    let verb = match command {
        Command::Node(c) => match c {
            NodeCmd::List => "node list",
            NodeCmd::Get { .. } => "node get",
            NodeCmd::SetVolume { .. } => "node set-volume",
            NodeCmd::Mute { .. } => "node mute",
            NodeCmd::SetDefault { .. } => "node set-default",
        },
        Command::Device(c) => match c {
            DeviceCmd::List => "device list",
            DeviceCmd::Get { .. } => "device get",
            DeviceCmd::SetProfile { .. } => "device set-profile",
        },
        Command::Link(LinkCmd::List) => "link list",
        Command::Metadata(c) => match c {
            MetadataCmd::List => "metadata list",
            MetadataCmd::Get { .. } => "metadata get",
        },
        Command::Server(ServerCmd::Info) => "server info",
        Command::Schema => "schema",
        Command::Describe => "describe",
    };
    format!("wiremix {verb}")
}

fn dispatch(
    command: Command,
    global: &GlobalArgs,
    config: &Config,
    command_str: &str,
) -> Result<Value, AppError> {
    match command {
        Command::Schema => Ok(schema::schema()),
        Command::Describe => Ok(schema::describe()),
        Command::Node(cmd) => node(cmd, global, config, command_str),
        Command::Device(cmd) => device(cmd, global, config, command_str),
        Command::Link(LinkCmd::List) => link_list(config, command_str),
        Command::Metadata(cmd) => metadata(cmd, config, command_str),
        Command::Server(ServerCmd::Info) => server_info(config, command_str),
    }
}

// --- Reads & writes: nodes -------------------------------------------------

fn node(
    cmd: NodeCmd,
    global: &GlobalArgs,
    config: &Config,
    command_str: &str,
) -> Result<Value, AppError> {
    match cmd {
        NodeCmd::List => {
            let conn = Connection::open(config.remote.clone())?;
            let view = build_view(&conn, config);
            let nodes: Vec<dto::NodeDto> = view
                .nodes_all
                .iter()
                .filter_map(|id| view.nodes.get(id))
                .map(|node| dto::NodeDto::from(&view, node))
                .collect();
            respond(command_str, nodes, false)
        }
        NodeCmd::Get { id } => {
            let conn = Connection::open(config.remote.clone())?;
            let view = build_view(&conn, config);
            let node = view.nodes.get(&id).ok_or_else(|| node_not_found(id))?;
            respond(command_str, dto::NodeDto::from(&view, node), false)
        }
        NodeCmd::SetVolume { id, percent } => {
            if global.dry_run {
                return dry_run(
                    command_str,
                    json!({
                        "action": "node set-volume",
                        "id": u32::from(id),
                        "percent": percent,
                    }),
                );
            }
            let mut conn = Connection::open(config.remote.clone())?;
            {
                let view = build_view(&conn, config);
                if !view.nodes.contains_key(&id) {
                    return Err(node_not_found(id));
                }
                let max = config
                    .enforce_max_volume
                    .then_some(config.max_volume_percent);
                let changed = view.volume(
                    id,
                    VolumeAdjustment::Absolute(percent / 100.0),
                    max,
                );
                if !changed {
                    return Err(AppError::new(
                        ErrorCode::Conflict,
                        format!(
                            "volume change rejected (no channels, or would \
                             exceed max-volume-percent {})",
                            config.max_volume_percent
                        ),
                    ));
                }
            }
            conn.settle();
            read_node(&conn, config, id, command_str)
        }
        NodeCmd::Mute { id, on, off, .. } => {
            if global.dry_run {
                let desired = mute_desired(on, off);
                return dry_run(
                    command_str,
                    json!({
                        "action": "node mute",
                        "id": u32::from(id),
                        "target": desired.map_or("toggle", |m| if m { "on" } else { "off" }),
                    }),
                );
            }
            let mut conn = Connection::open(config.remote.clone())?;
            {
                let view = build_view(&conn, config);
                let node =
                    view.nodes.get(&id).ok_or_else(|| node_not_found(id))?;
                let should_toggle = match mute_desired(on, off) {
                    Some(desired) => node.mute != desired,
                    None => true,
                };
                if should_toggle {
                    view.mute(id);
                }
            }
            conn.settle();
            read_node(&conn, config, id, command_str)
        }
        NodeCmd::SetDefault { id } => {
            if global.dry_run {
                return dry_run(
                    command_str,
                    json!({
                        "action": "node set-default",
                        "id": u32::from(id),
                    }),
                );
            }
            let mut conn = Connection::open(config.remote.clone())?;
            {
                let view = build_view(&conn, config);
                let node =
                    view.nodes.get(&id).ok_or_else(|| node_not_found(id))?;
                let kind = if media_class::is_sink(&node.media_class) {
                    DeviceKind::Sink
                } else if media_class::is_source(&node.media_class) {
                    DeviceKind::Source
                } else {
                    return Err(AppError::usage(format!(
                        "node {} is not a sink or source; only devices can be \
                         set as default",
                        u32::from(id)
                    ))
                    .with_hint("wiremix node list --json"));
                };
                view.set_default(id, kind);
            }
            conn.settle();
            read_node(&conn, config, id, command_str)
        }
    }
}

fn read_node(
    conn: &Connection,
    config: &Config,
    id: ObjectId,
    command_str: &str,
) -> Result<Value, AppError> {
    let view = build_view(conn, config);
    let node = view.nodes.get(&id).ok_or_else(|| node_not_found(id))?;
    respond(command_str, dto::NodeDto::from(&view, node), false)
}

// --- Reads & writes: devices ----------------------------------------------

fn device(
    cmd: DeviceCmd,
    global: &GlobalArgs,
    config: &Config,
    command_str: &str,
) -> Result<Value, AppError> {
    match cmd {
        DeviceCmd::List => {
            let conn = Connection::open(config.remote.clone())?;
            let view = build_view(&conn, config);
            let devices: Vec<dto::DeviceDto> = view
                .devices_all
                .iter()
                .filter_map(|id| view.devices.get(id))
                .map(dto::DeviceDto::from)
                .collect();
            respond(command_str, devices, false)
        }
        DeviceCmd::Get { id } => {
            let conn = Connection::open(config.remote.clone())?;
            let view = build_view(&conn, config);
            let device =
                view.devices.get(&id).ok_or_else(|| device_not_found(id))?;
            respond(command_str, dto::DeviceDto::from(device), false)
        }
        DeviceCmd::SetProfile { id, profile } => {
            if global.dry_run {
                return dry_run(
                    command_str,
                    json!({
                        "action": "device set-profile",
                        "id": u32::from(id),
                        "profile": profile,
                    }),
                );
            }
            let mut conn = Connection::open(config.remote.clone())?;
            {
                let view = build_view(&conn, config);
                let device = view
                    .devices
                    .get(&id)
                    .ok_or_else(|| device_not_found(id))?;
                let known = device.profiles.iter().any(|(target, _)| {
                    matches!(target, crate::view::Target::Profile(_, index)
                        if *index == profile)
                });
                if !known {
                    return Err(AppError::not_found(format!(
                        "device {} has no profile index {profile}",
                        u32::from(id)
                    ))
                    .with_hint(format!(
                        "wiremix device get {} --json",
                        u32::from(id)
                    )));
                }
            }
            conn.sender().device_set_profile(id, profile);
            conn.settle();
            let view = build_view(&conn, config);
            let device =
                view.devices.get(&id).ok_or_else(|| device_not_found(id))?;
            respond(command_str, dto::DeviceDto::from(device), false)
        }
    }
}

// --- Reads: links, metadata, server ---------------------------------------

fn link_list(config: &Config, command_str: &str) -> Result<Value, AppError> {
    let conn = Connection::open(config.remote.clone())?;
    let links: Vec<dto::LinkDto> = conn
        .state()
        .links
        .iter()
        .map(|(id, link)| dto::LinkDto {
            id: u32::from(*id),
            output_id: link.output_id.into(),
            input_id: link.input_id.into(),
        })
        .collect();
    respond(command_str, links, false)
}

fn metadata(
    cmd: MetadataCmd,
    config: &Config,
    command_str: &str,
) -> Result<Value, AppError> {
    let conn = Connection::open(config.remote.clone())?;
    match cmd {
        MetadataCmd::List => {
            let items: Vec<dto::MetadataDto> = conn
                .state()
                .metadatas
                .values()
                .map(dto::MetadataDto::from)
                .collect();
            respond(command_str, items, false)
        }
        MetadataCmd::Get { name } => {
            let metadata =
                conn.state().get_metadata_by_name(&name).ok_or_else(|| {
                    AppError::not_found(format!("no metadata named {name:?}"))
                        .with_hint("wiremix metadata list --json")
                })?;
            respond(command_str, dto::MetadataDto::from(metadata), false)
        }
    }
}

fn server_info(config: &Config, command_str: &str) -> Result<Value, AppError> {
    let conn = Connection::open(config.remote.clone())?;
    let view = build_view(&conn, config);
    let info =
        dto::ServerInfoDto::from(&view, conn.state(), config.remote.clone());
    respond(command_str, info, true)
}

// --- Helpers ---------------------------------------------------------------

fn build_view<'a>(conn: &'a Connection, config: &Config) -> View<'a> {
    // The CLI shows the full graph: filters are a TUI display preference.
    View::from(conn.sender(), conn.state(), &config.names, &[])
}

fn mute_desired(on: bool, off: bool) -> Option<bool> {
    match (on, off) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    }
}

fn node_not_found(id: ObjectId) -> AppError {
    AppError::not_found(format!("no node with id {}", u32::from(id)))
        .with_hint("wiremix node list --json")
}

fn device_not_found(id: ObjectId) -> AppError {
    AppError::not_found(format!("no device with id {}", u32::from(id)))
        .with_hint("wiremix device list --json")
}

fn respond<T: Serialize>(
    command: &str,
    data: T,
    attribution: bool,
) -> Result<Value, AppError> {
    let response = envelope::Response::new(command, data);
    let response = if attribution {
        response.with_attribution()
    } else {
        response
    };
    serde_json::to_value(response).map_err(|e| {
        AppError::new(ErrorCode::Internal, format!("serialization failed: {e}"))
    })
}

fn dry_run(command: &str, plan: Value) -> Result<Value, AppError> {
    respond(command, json!({ "dry_run": true, "plan": plan }), false)
}

fn emit_ok(mut value: Value, mode: Mode, global: &GlobalArgs) -> ExitCode {
    if let Some(fields) = &global.fields {
        if let Some(data) = value.get_mut("data") {
            output::project_fields(data, fields);
        }
    }

    let terminator = if global.print0 { '\0' } else { '\n' };
    let mut stdout = std::io::stdout();
    match mode {
        Mode::Json => {
            let _ = write!(stdout, "{}{terminator}", compact(&value));
        }
        Mode::Human => {
            let pretty = serde_json::to_string_pretty(&value)
                .unwrap_or_else(|_| compact(&value));
            let _ = write!(stdout, "{pretty}{terminator}");
        }
        Mode::Jsonl => match value.get("data") {
            Some(Value::Array(items)) => {
                for item in items {
                    let _ = write!(stdout, "{}{terminator}", compact(item));
                }
            }
            Some(other) => {
                let _ = write!(stdout, "{}{terminator}", compact(other));
            }
            None => {
                let _ = write!(stdout, "{}{terminator}", compact(&value));
            }
        },
    }
    let _ = stdout.flush();
    ExitCode::Success
}

fn emit_error(err: &AppError, mode: Mode, command: &str) -> ExitCode {
    match mode {
        Mode::Human => {
            eprintln!("error: {}", err.message);
            if let Some(hint) = &err.hint {
                eprintln!("hint: {hint}");
            }
        }
        Mode::Json | Mode::Jsonl => {
            let object = json!({
                "error": {
                    "code": err.code,
                    "exit_code": err.exit_code().code(),
                    "message": err.message,
                    "hint": err.hint,
                    "timestamp": envelope::now(),
                    "command": command,
                    "docs_url": DOCS_URL,
                }
            });
            eprintln!("{}", compact(&object));
        }
    }
    err.exit_code()
}

fn compact(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| String::from("{}"))
}
