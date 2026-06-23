// SPDX-FileCopyrightText: 2025-2026 Thomas Sowell <tom@ldtlb.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Type representing whether a device is sink or source.

#[derive(Debug, Clone, Copy)]
pub enum DeviceKind {
    Sink,
    Source,
}
