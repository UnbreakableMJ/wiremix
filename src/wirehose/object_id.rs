// SPDX-FileCopyrightText: 2025-2026 Thomas Sowell <tom@ldtlb.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Type for representing PipeWire object IDs.

use libspa::utils::dict::DictRef;
use pipewire::registry::GlobalObject;

/// A PipeWire object ID.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectId(u32);

impl From<&GlobalObject<&DictRef>> for ObjectId {
    fn from(obj: &GlobalObject<&DictRef>) -> Self {
        ObjectId(obj.id)
    }
}

impl From<ObjectId> for u32 {
    fn from(id: ObjectId) -> u32 {
        id.0
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ObjectId {
    pub fn from_raw_id(id: u32) -> Self {
        ObjectId(id)
    }
}
