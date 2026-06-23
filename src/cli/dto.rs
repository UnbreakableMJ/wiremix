// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stable, snake_case data-transfer objects for machine output.
//!
//! These project the internal `view`/`state` structures into a documented,
//! version-stable JSON shape. Internal structs are never serialized directly.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::view::{Target, View};
use crate::wirehose::state::State;
use crate::wirehose::ObjectId;

/// Convert a raw PipeWire channel volume to a display percentage, using the
/// same cubic mapping as the TUI (`display% = cbrt(raw) * 100`).
fn percent(raw: f32) -> u32 {
    (raw.cbrt() * 100.0).round().max(0.0) as u32
}

fn mean_percent(volumes: &[f32]) -> u32 {
    if volumes.is_empty() {
        return 0;
    }
    let mean = volumes.iter().sum::<f32>() / volumes.len() as f32;
    percent(mean)
}

/// The tab/category a controllable node belongs to.
pub fn node_kind(view: &View, id: ObjectId) -> &'static str {
    if view.nodes_playback.contains(&id) {
        "playback"
    } else if view.nodes_recording.contains(&id) {
        "recording"
    } else if view.nodes_output.contains(&id) {
        "output"
    } else if view.nodes_input.contains(&id) {
        "input"
    } else {
        "other"
    }
}

#[derive(Debug, Serialize)]
pub struct NodeDto {
    pub id: u32,
    pub serial: u64,
    pub name: String,
    pub title: String,
    pub media_class: String,
    pub kind: &'static str,
    pub volume_percent: u32,
    pub channel_volumes_percent: Vec<u32>,
    pub channels: usize,
    pub muted: bool,
    pub is_default_sink: bool,
    pub is_default_source: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<u32>,
}

impl NodeDto {
    pub fn from(view: &View, node: &crate::view::Node) -> Self {
        Self {
            id: node.object_id.into(),
            serial: node.object_serial,
            name: node.name.clone(),
            title: node.title.clone(),
            media_class: node.media_class.clone(),
            kind: node_kind(view, node.object_id),
            volume_percent: mean_percent(&node.volumes),
            channel_volumes_percent: node
                .volumes
                .iter()
                .map(|&raw| percent(raw))
                .collect(),
            channels: node.volumes.len(),
            muted: node.mute,
            is_default_sink: node.is_default_sink,
            is_default_source: node.is_default_source,
            target: node.target_title.clone(),
            client_id: node.client_id.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProfileDto {
    pub index: i32,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceDto {
    pub id: u32,
    pub serial: u64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile_index: Option<i32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub active_profile: String,
    pub profiles: Vec<ProfileDto>,
}

impl DeviceDto {
    pub fn from(device: &crate::view::Device) -> Self {
        let active_profile_index = match device.target {
            Some(Target::Profile(_, index)) => Some(index),
            _ => None,
        };
        let profiles = device
            .profiles
            .iter()
            .filter_map(|(target, description)| match target {
                Target::Profile(_, index) => Some(ProfileDto {
                    index: *index,
                    description: description.clone(),
                }),
                _ => None,
            })
            .collect();
        Self {
            id: device.object_id.into(),
            serial: device.object_serial,
            title: device.title.clone(),
            active_profile_index,
            active_profile: device.target_title.clone(),
            profiles,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LinkDto {
    pub id: u32,
    pub output_id: u32,
    pub input_id: u32,
}

#[derive(Debug, Serialize)]
pub struct MetadataDto {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Properties keyed by subject id, then property key.
    pub properties: BTreeMap<u32, BTreeMap<String, String>>,
}

impl MetadataDto {
    pub fn from(metadata: &crate::wirehose::state::Metadata) -> Self {
        let properties = metadata
            .properties
            .iter()
            .map(|(subject, props)| {
                (
                    *subject,
                    props.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                )
            })
            .collect();
        Self {
            id: metadata.object_id.into(),
            name: metadata.metadata_name.clone(),
            properties,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NodeRef {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Counts {
    pub nodes: usize,
    pub devices: usize,
    pub links: usize,
    pub clients: usize,
    pub metadata: usize,
}

#[derive(Debug, Serialize)]
pub struct ServerInfoDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_sink: Option<NodeRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_source: Option<NodeRef>,
    pub counts: Counts,
}

impl ServerInfoDto {
    pub fn from(view: &View, state: &State, remote: Option<String>) -> Self {
        let node_ref = |target: Option<Target>| -> Option<NodeRef> {
            let id = target?.object_id()?;
            view.nodes.get(&id).map(|node| NodeRef {
                id: id.into(),
                name: node.name.clone(),
            })
        };
        Self {
            remote,
            default_sink: node_ref(view.default_sink),
            default_source: node_ref(view.default_source),
            counts: Counts {
                nodes: state.nodes.len(),
                devices: state.devices.len(),
                links: state.links.len(),
                clients: state.clients.len(),
                metadata: state.metadatas.len(),
            },
        }
    }
}
