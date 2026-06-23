// SPDX-FileCopyrightText: 2025-2026 Thomas Sowell <tom@ldtlb.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

use libspa::pod::{deserialize::PodDeserializer, Object, Pod, Value};

pub fn deserialize(param: Option<&Pod>) -> Option<Object> {
    param
        .and_then(|pod| {
            PodDeserializer::deserialize_any_from(pod.as_bytes()).ok()
        })
        .and_then(|(_, value)| match value {
            Value::Object(obj) => Some(obj),
            _ => None,
        })
}
