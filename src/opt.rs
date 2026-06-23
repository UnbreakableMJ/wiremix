// SPDX-FileCopyrightText: 2025-2026 Thomas Sowell <tom@ldtlb.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Parse command-line arguments.

use std::path::PathBuf;

use clap::Parser;

use crate::config::{self, TabKind};

const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

// `--version` carries the §15.2 attribution block; `--help` gets the footer.
const LONG_VERSION: &str = concat!(
    "v",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>\n",
    "Copyright (C) 2026 Mohamed Hammad & Spacecraft Software\n",
    "License: GPL-3.0-or-later\n",
    "https://Wiremix.SpacecraftSoftware.org/",
);
const FOOTER: &str = concat!(
    "Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>\n",
    "https://Wiremix.SpacecraftSoftware.org/",
);

#[derive(Parser, Default)]
#[clap(
    name = "wiremix",
    about = "Dual-mode (TUI + agent-native CLI) mixer for PipeWire"
)]
#[command(version = VERSION, long_version = LONG_VERSION, after_help = FOOTER)]
pub struct Opt {
    /// Run a one-shot CLI command (omit for the interactive TUI)
    #[command(subcommand)]
    pub command: Option<crate::cli::Command>,

    /// Global CLI flags (--json, --format, --fields, --dry-run, ...)
    #[command(flatten)]
    pub global: crate::cli::GlobalArgs,

    /// Override default config file path
    #[clap(short = 'c', long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// The name of the remote to connect to
    #[clap(short, long, value_name = "NAME", global = true)]
    pub remote: Option<String>,

    /// Target frames per second (or 0 for unlimited)
    #[clap(short, long)]
    pub fps: Option<f32>,

    /// Character set to use [built-in sets: default, compat, extracompat]
    #[clap(short = 's', long, value_name = "NAME")]
    pub char_set: Option<String>,

    /// Theme to use [built-in themes: default, nocolor, plain]
    #[clap(short, long, value_name = "NAME")]
    pub theme: Option<String>,

    /// Audio peak meters
    #[clap(short, long, value_parser = clap::value_parser!(config::Peaks))]
    pub peaks: Option<config::Peaks>,

    /// Disable mouse support
    #[clap(long, conflicts_with = "mouse")]
    pub no_mouse: bool,

    /// Enable mouse support
    #[clap(long, conflicts_with = "no_mouse")]
    pub mouse: bool,

    /// Initial tab view
    #[clap(
        short = 'v',
        long,
        value_enum,
        value_parser = clap::value_parser!(TabKind),
    )]
    pub tab: Option<TabKind>,

    /// Which tabs are present and their order
    #[clap(short = 'T', long, num_args = 1.., value_enum)]
    pub tabs: Option<Vec<TabKind>>,

    /// Maximum volume for volume sliders
    #[clap(short = 'm', long, value_name = "PERCENT")]
    pub max_volume_percent: Option<f32>,

    /// Allow increasing volume past max-volume-percent
    #[clap(long, conflicts_with = "enforce_max_volume")]
    pub no_enforce_max_volume: bool,

    /// Prevent increasing volume past max-volume-percent
    #[clap(long, conflicts_with = "no_enforce_max_volume")]
    pub enforce_max_volume: bool,

    /// Monitor peak levels of all nodes
    #[clap(long, conflicts_with = "lazy_capture")]
    pub no_lazy_capture: bool,

    /// Only monitor peak levels of on-screen nodes (reduces CPU usage, but
    /// peaks appear with a slight delay)
    #[clap(long, conflicts_with = "no_lazy_capture")]
    pub lazy_capture: bool,

    #[cfg(debug_assertions)]
    #[clap(short, long)]
    pub dump_events: bool,
}

impl Opt {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
