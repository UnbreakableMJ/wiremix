// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `wiremix schema` and `wiremix describe` introspection (CLI Standard §2
//! Rule 4, Agentic CLI §5). Both are generated from one command table so they
//! never drift apart.

use serde_json::{json, Map, Value};

use crate::cli::{DOCS_URL, MAINTAINER, TOOL, VERSION, WEBSITE};

struct Param {
    name: &'static str,
    /// JSON Schema primitive type.
    ty: &'static str,
    required: bool,
    description: &'static str,
}

struct Spec {
    name: &'static str,
    summary: &'static str,
    /// `read` | `write` | `meta`.
    kind: &'static str,
    params: &'static [Param],
    /// A complete, runnable invocation (Agentic CLI §5: examples must be
    /// accurate, not illustrative).
    example: &'static str,
}

const COMMANDS: &[Spec] = &[
    Spec {
        name: "node list",
        summary: "List controllable nodes (streams and device endpoints).",
        kind: "read",
        params: &[],
        example: "wiremix node list --json",
    },
    Spec {
        name: "node get",
        summary: "Show one node: volumes, mute, default flags, target.",
        kind: "read",
        params: &[Param {
            name: "id",
            ty: "integer",
            required: true,
            description: "PipeWire object id (see `node list`).",
        }],
        example: "wiremix node get 42 --json",
    },
    Spec {
        name: "node set-volume",
        summary: "Set a node's volume (percent, applied to all channels).",
        kind: "write",
        params: &[
            Param {
                name: "id",
                ty: "integer",
                required: true,
                description: "PipeWire object id.",
            },
            Param {
                name: "percent",
                ty: "number",
                required: true,
                description: "Target volume as a percentage (100 = unity).",
            },
        ],
        example: "wiremix node set-volume 42 50",
    },
    Spec {
        name: "node mute",
        summary: "Mute, unmute, or toggle a node (default: toggle).",
        kind: "write",
        params: &[
            Param {
                name: "id",
                ty: "integer",
                required: true,
                description: "PipeWire object id.",
            },
            Param {
                name: "on",
                ty: "boolean",
                required: false,
                description: "Mute the node.",
            },
            Param {
                name: "off",
                ty: "boolean",
                required: false,
                description: "Unmute the node.",
            },
            Param {
                name: "toggle",
                ty: "boolean",
                required: false,
                description: "Toggle mute (the default if none given).",
            },
        ],
        example: "wiremix node mute 42 --toggle",
    },
    Spec {
        name: "node set-default",
        summary: "Set a node as the default sink or source.",
        kind: "write",
        params: &[Param {
            name: "id",
            ty: "integer",
            required: true,
            description: "PipeWire object id of a sink or source node.",
        }],
        example: "wiremix node set-default 42",
    },
    Spec {
        name: "device list",
        summary: "List devices with their profiles.",
        kind: "read",
        params: &[],
        example: "wiremix device list --json",
    },
    Spec {
        name: "device get",
        summary: "Show one device: active profile and available profiles.",
        kind: "read",
        params: &[Param {
            name: "id",
            ty: "integer",
            required: true,
            description: "PipeWire object id (see `device list`).",
        }],
        example: "wiremix device get 50 --json",
    },
    Spec {
        name: "device set-profile",
        summary: "Switch a device to a profile by index.",
        kind: "write",
        params: &[
            Param {
                name: "id",
                ty: "integer",
                required: true,
                description: "PipeWire object id.",
            },
            Param {
                name: "profile",
                ty: "integer",
                required: true,
                description: "Profile index (see `device get`).",
            },
        ],
        example: "wiremix device set-profile 50 1",
    },
    Spec {
        name: "link list",
        summary: "List links (connections) between nodes.",
        kind: "read",
        params: &[],
        example: "wiremix link list --json",
    },
    Spec {
        name: "metadata list",
        summary: "List metadata objects and their properties.",
        kind: "read",
        params: &[],
        example: "wiremix metadata list --json",
    },
    Spec {
        name: "metadata get",
        summary: "Show one metadata object by name (e.g. \"default\").",
        kind: "read",
        params: &[Param {
            name: "name",
            ty: "string",
            required: false,
            description: "Metadata name; defaults to \"default\".",
        }],
        example: "wiremix metadata get --name default --json",
    },
    Spec {
        name: "server info",
        summary: "Default sink/source, object counts, and remote.",
        kind: "read",
        params: &[],
        example: "wiremix server info --json",
    },
    Spec {
        name: "schema",
        summary: "Print this JSON Schema of the CLI.",
        kind: "meta",
        params: &[],
        example: "wiremix schema",
    },
    Spec {
        name: "describe",
        summary: "Print a capability manifest of the CLI.",
        kind: "meta",
        params: &[],
        example: "wiremix describe",
    },
];

const GLOBAL_FLAGS: &[(&str, &str)] = &[
    ("--json", "Alias for --format json."),
    (
        "--format <json|jsonl|explore>",
        "Output format (explore = TUI).",
    ),
    (
        "--fields <a,b,...>",
        "Restrict output records to these fields.",
    ),
    (
        "--dry-run",
        "Emit the action plan as JSON; make no changes.",
    ),
    ("--verbose", "Diagnostic output to stderr."),
    ("--quiet / -q", "Suppress non-error stderr."),
    (
        "--color <never|always|auto>",
        "Color control (output is plain text).",
    ),
    ("--no-color", "Disable color."),
    (
        "--absolute-time",
        "Render absolute time (output is always UTC).",
    ),
    ("--print0 / -0", "NUL-terminate output for safe piping."),
    ("--yes / --force", "Skip confirmation in non-TTY mode."),
];

fn exit_codes() -> Value {
    json!({
        "0": "success",
        "1": "general failure",
        "2": "usage error",
        "3": "not found",
        "4": "permission denied",
        "5": "conflict",
    })
}

fn params_schema(params: &[Param]) -> (Value, Vec<&'static str>) {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for param in params {
        properties.insert(
            param.name.to_string(),
            json!({ "type": param.ty, "description": param.description }),
        );
        if param.required {
            required.push(param.name);
        }
    }
    (Value::Object(properties), required)
}

/// The `wiremix schema` document: a JSON Schema (Draft 2020-12) whose `oneOf`
/// branches describe each command's parameters, keyed by a `command` const.
pub fn schema() -> Value {
    let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();

    let branches: Vec<Value> = COMMANDS
        .iter()
        .map(|spec| {
            let (mut properties, mut required) = {
                let (props, req) = params_schema(spec.params);
                let map = match props {
                    Value::Object(map) => map,
                    _ => Map::new(),
                };
                (map, req.into_iter().map(String::from).collect::<Vec<_>>())
            };
            properties
                .insert("command".to_string(), json!({ "const": spec.name }));
            required.insert(0, "command".to_string());
            json!({
                "title": spec.name,
                "description": spec.summary,
                "type": "object",
                "properties": properties,
                "required": required,
                "additionalProperties": false,
            })
        })
        .collect();

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://Wiremix.SpacecraftSoftware.org/schema/cli.json",
        "title": "wiremix CLI",
        "description": "Command surface of the wiremix dual-mode PipeWire mixer.",
        "type": "object",
        "required": ["command"],
        "properties": { "command": { "enum": names } },
        "oneOf": branches,
        "x-exit-codes": exit_codes(),
    })
}

/// The `wiremix describe` manifest: a human/agent-readable capability surface.
pub fn describe() -> Value {
    let commands: Vec<Value> = COMMANDS
        .iter()
        .map(|spec| {
            let params: Vec<Value> = spec
                .params
                .iter()
                .map(|param| {
                    json!({
                        "name": param.name,
                        "type": param.ty,
                        "required": param.required,
                        "description": param.description,
                    })
                })
                .collect();
            json!({
                "name": spec.name,
                "summary": spec.summary,
                "kind": spec.kind,
                "params": params,
                "example": spec.example,
            })
        })
        .collect();

    let global_flags: Vec<Value> = GLOBAL_FLAGS
        .iter()
        .map(|(flag, description)| json!({ "flag": flag, "description": description }))
        .collect();

    json!({
        "tool": TOOL,
        "version": VERSION,
        "maintainer": MAINTAINER,
        "website": WEBSITE,
        "docs_url": DOCS_URL,
        "summary": "Dual-mode (TUI + agent-native CLI) mixer for PipeWire.",
        "global_flags": global_flags,
        "exit_codes": exit_codes(),
        "commands": commands,
    })
}
