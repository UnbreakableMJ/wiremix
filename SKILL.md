---
name: wiremix
description: >-
  Adjust PipeWire audio from the command line or scripts: list and inspect
  nodes/devices/links, set volumes, mute/unmute, set default sink/source, and
  switch device profiles/routes. Dual-mode — interactive TUI for humans, JSON
  noun-verb CLI for agents. Use when a task involves changing or querying
  PipeWire audio state (volume, routing, mute, default device, profile).
license: GPL-3.0-or-later
---

# wiremix

A dual-mode mixer for PipeWire. Humans get an interactive TUI; agents and
scripts get a self-documenting noun-verb CLI with stable JSON output.

## Invocation & modes

- `wiremix` (TTY) → interactive TUI. `wiremix --format explore` forces it.
- `wiremix <noun> <verb> [--json]` → one-shot machine command.
- Output auto-detects: piped / `--json` / `AI_AGENT|AGENT|CI` set → JSON;
  TTY → human. stdout = data only; errors are structured JSON on stderr with a
  runnable `hint`. Exit codes: 0 ok, 1 general, 2 usage, 3 not-found,
  4 permission, 5 conflict.

## Capability surface

| Command | Kind | Effect |
|---------|------|--------|
| `node list` | read | List playback/recording/output/input nodes |
| `node get <id>` | read | One node: name, volumes, mute, peaks, media class |
| `node set-volume <id> <pct...>` | write | Set per-channel volume (percent) |
| `node mute <id> [--on\|--off\|--toggle]` | write | Mute/unmute a node |
| `node set-default <id>` | write | Set default sink/source |
| `device list` / `device get <id>` | read | Devices, profiles, routes |
| `device set-profile <id> <index>` | write | Switch device profile |
| `device set-route <id> --route <i> --device <d>` | write | Switch route (port) |
| `device set-volume <id> --route <i> --device <d> <pct...>` | write | Device route volume |
| `device mute <id> --route <i> --device <d> [--on\|--off]` | write | Mute a device route |
| `link list` | read | PipeWire links (connections) |
| `metadata list` / `metadata get [--name default]` | read | Metadata (e.g. defaults) |
| `server info` | read | Default sink/source, object counts, remote |
| `schema` / `describe` | meta | JSON Schema (Draft 2020-12) / capability manifest |

Every write supports `--dry-run` (emits the action plan as JSON, no side
effects). Global flags: `--json`, `--format`, `--fields`, `--color`,
`--no-color`, `--quiet`, `--verbose`, `--absolute-time`, `--print0`,
`--yes/--force`. Get the authoritative, machine-readable contract from
`wiremix schema`.

## Notes for agents

- Object IDs are PipeWire IDs and are **not stable across restarts**; `list`/
  `get` also surface `object.serial` and names — re-resolve targets by name when
  an ID misses (exit 3).
- Volumes are percent (`100` = unity gain); the CLI uses the same volume mapping
  as the TUI.
- Requires a running PipeWire session.

---

Maintained by Mohamed Hammad &lt;Mohamed.Hammad@SpacecraftSoftware.org&gt; ·
<https://Wiremix.SpacecraftSoftware.org/>
