// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the CLI's pure logic (no PipeWire connection required).

use serde_json::json;

use wiremix::cli::envelope::{self, Response};
use wiremix::cli::error::{AppError, ErrorCode, ExitCode};
use wiremix::cli::output::{self, Format, Mode, Resolved};
use wiremix::cli::schema;

#[test]
fn resolve_explicit_json_wins() {
    assert_eq!(output::resolve(true, None), Resolved::Output(Mode::Json));
}

#[test]
fn resolve_explicit_format() {
    assert_eq!(
        output::resolve(false, Some(Format::Jsonl)),
        Resolved::Output(Mode::Jsonl)
    );
    assert_eq!(output::resolve(false, Some(Format::Explore)), Resolved::Tui);
}

#[test]
fn fields_projection_keeps_only_requested() {
    let mut value = json!([
        { "a": 1, "b": 2, "c": 3 },
        { "a": 4, "b": 5 },
    ]);
    output::project_fields(&mut value, &[String::from("a"), String::from("c")]);
    assert_eq!(value, json!([{ "a": 1, "c": 3 }, { "a": 4 }]));
}

#[test]
fn exit_code_mapping() {
    assert_eq!(ExitCode::NotFound.code(), 3);
    assert_eq!(ErrorCode::NotFound.exit(), ExitCode::NotFound);
    assert_eq!(AppError::not_found("x").exit_code().code(), 3);
    assert_eq!(AppError::usage("x").exit_code().code(), 2);
    assert_eq!(AppError::unavailable("x").exit_code().code(), 1);
}

#[test]
fn timestamp_is_iso8601_utc_seconds() {
    let stamp = envelope::now();
    assert_eq!(
        stamp.len(),
        20,
        "expected YYYY-MM-DDTHH:MM:SSZ, got {stamp}"
    );
    assert!(stamp.ends_with('Z'));
    assert!(stamp.contains('T'));
}

#[test]
fn envelope_attribution() {
    let response = Response::new("wiremix server info", json!({ "ok": true }))
        .with_attribution();
    assert_eq!(response.metadata.tool, "wiremix");
    assert!(response.metadata.maintainer.is_some());
    assert!(response.metadata.website.is_some());
}

#[test]
fn schema_is_draft_2020_12() {
    let schema = schema::schema();
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["oneOf"].as_array().expect("oneOf array").len(), 14);
}

#[test]
fn describe_lists_every_command() {
    let describe = schema::describe();
    assert_eq!(describe["tool"], "wiremix");
    assert_eq!(
        describe["commands"]
            .as_array()
            .expect("commands array")
            .len(),
        14
    );
}
