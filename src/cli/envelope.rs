// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The JSON output envelope (CLI Standard §6).

use serde::Serialize;

use crate::cli::{MAINTAINER, TOOL, VERSION, WEBSITE};

/// Current UTC time as an ISO 8601 string with a `Z` suffix and second
/// precision (Standard §14). Uses `jiff`, the Standard-preferred time crate.
pub fn now() -> String {
    jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// The `metadata` block carried by every JSON response.
#[derive(Debug, Serialize)]
pub struct Metadata {
    pub tool: &'static str,
    pub version: &'static str,
    pub command: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<&'static str>,
}

/// A complete JSON response: `{ "metadata": {...}, "data": ... }`.
#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub metadata: Metadata,
    pub data: T,
}

impl<T> Response<T> {
    /// Build a response for `command`, stamping tool/version/timestamp.
    pub fn new(command: impl Into<String>, data: T) -> Self {
        Self {
            metadata: Metadata {
                tool: TOOL,
                version: VERSION,
                command: command.into(),
                timestamp: now(),
                maintainer: None,
                website: None,
            },
            data,
        }
    }

    /// Add maintainer + website to the metadata (Standard §15.2). Used by
    /// `server info`, `describe`, and version output.
    pub fn with_attribution(mut self) -> Self {
        self.metadata.maintainer = Some(MAINTAINER);
        self.metadata.website = Some(WEBSITE);
        self
    }
}
