# AGENTS.md

Project-specific invariants for AI agents and automated tooling working in this
repository. General Spacecraft Software conventions live in the `spacecraft-*`
skills; this file is only what is special about **wiremix**.

## What this is

A dual-mode PipeWire mixer: an interactive ratatui **TUI** and a machine-readable
**CLI**, plus an optional Slint **desktop GUI** (`wiremix-gui`, `gui` feature).
It is a `GPL-3.0-or-later` fork of upstream `wiremix` (MIT/Apache-2.0,
Thomas Sowell). Rust, MSRV 1.74, edition 2021 — the `gui` feature needs Rust
≥ 1.88 (current Slint), but the default TUI/CLI MSRV is unchanged.

## Build / test / lint (run inside `nix develop`)

The build needs PipeWire dev libraries + `pkg-config` + a bindgen-capable
toolchain. `nix develop` provides them; outside it the build fails at
`libspa-sys`.

```sh
nix develop -c cargo build --locked --all-features --all-targets
nix develop -c cargo test  --locked --all-features --all-targets
nix develop -c cargo clippy --locked --all-features --all-targets -- -D warnings
nix develop -c cargo fmt --all --check          # rustfmt max_width = 80
nix develop -c reuse lint                        # REUSE/SPDX compliance

# Optional desktop GUI (not in default build; needs Rust >= 1.88):
nix develop -c cargo build --locked --features gui --bin wiremix-gui
```

A single test: `cargo test <name>` (tests are inline `#[cfg(test)] mod tests`,
no `tests/` dir). The TUI cannot be tested headless — `App` is tested against a
mock `CommandSender` (`src/lib.rs` `mod mock`).

## Invariants — do not break these

- **`wirehose` is the only code that touches PipeWire.** Everything else
  (TUI, CLI) goes through its public API (`Session`, `CommandSender`, `State`,
  `StateEvent`, `PropertyStore`, `ObjectId`). Do not call `pipewire`/`libspa`
  crates outside `src/wirehose/`.
- **The TUI path stays untouched by CLI work.** The CLI is a separate one-shot
  path under `src/cli/`; `src/app.rs` (the TUI loop) should not gain CLI logic.
- **The GUI is a third frontend, not a fork of the core.** `src/gui/` + `ui/`
  drive the same `wirehose` `Session` and `view::View` as the TUI/CLI (commands
  via `View`/`CommandSender`, never `pipewire`/`libspa` directly). Keep it
  behind the `gui` feature so the default build stays Slint-free; do not let
  GUI logic leak into `src/app.rs` or `src/cli/`.
- **Dual-mode output contract** (the reason this CLI exists — keep it exact):
  - stdout = data only; stderr = logs, progress, diagnostics, structured errors.
    Never mix them. No ANSI escapes in machine mode. UTF-8, no BOM.
  - Every data-returning subcommand supports `--json` and emits the
    `{ metadata, data }` envelope; keys snake_case; null fields omitted.
  - Canonical exit codes (0 ok, 1 general, 2 usage, 3 not-found, 4 perm,
    5 conflict). Non-zero in machine mode → structured error on stderr with a
    **runnable** `hint`.
  - Output mode cascade: explicit `--format`/`--json` → `AI_AGENT`/`AGENT`/`CI`
    → stdout isatty → non-TTY → fallback. `--format explore` is the TUI and must
    NOT activate under `AI_AGENT`/`AGENT`.
  - Timestamps ISO 8601 UTC with `Z`. Durations ISO 8601. Metric/24h only.
- **REUSE/SPDX on every file.** New files: `GPL-3.0-or-later` + Mohamed Hammad
  copyright. Do not strip upstream `MIT OR Apache-2.0` headers from files you
  don't substantially rewrite. `reuse lint` must pass.
- **Commits are signed** (Standard §6.3) — never `--no-gpg-sign`.

## Layout pointers

- `src/wirehose/` — PipeWire wrapper (Session/State/commands/events).
- `src/cli/` — one-shot machine CLI (output/envelope/error/dto/oneshot/schema).
- `src/app.rs` + widgets — the TUI.
- `src/gui/` + `ui/main.slint` — optional Slint desktop GUI (`gui` feature);
  `build.rs` compiles the `.slint` only when that feature is on.
- `src/config/` — TOML config, themes (incl. `steelbore`), char-sets, keybinds.
- `wiremix.toml` — bundled example + canonical config reference; keep it in sync.
- `doc/wiremix.texi` — the manual (canonical CLI/option reference).
