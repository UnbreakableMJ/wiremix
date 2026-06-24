// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Build script. Compiles the Slint UI only when the `gui` feature is enabled,
//! so the default TUI/CLI build pulls in neither Slint nor a graphics stack.

fn main() {
    #[cfg(feature = "gui")]
    slint_build::compile("ui/main.slint").expect("failed to compile Slint UI");
}
