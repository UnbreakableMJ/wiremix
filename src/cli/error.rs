// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Canonical exit codes and structured CLI errors.
//!
//! Implements the Spacecraft Software CLI Standard §4 (exit-code map) and the
//! Agentic CLI "tips-thinking" discipline: every error carries a `hint` that is
//! a *runnable* next command, not prose.

use std::fmt;

use serde::Serialize;

/// Canonical process exit codes (CLI Standard §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Failure = 1,
    Usage = 2,
    NotFound = 3,
    Permission = 4,
    Conflict = 5,
}

impl ExitCode {
    /// The numeric code to pass to [`std::process::exit`].
    pub fn code(self) -> i32 {
        self as i32
    }
}

/// Stable machine-readable error identifier (the `error.code` field, §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Usage,
    NotFound,
    Permission,
    Conflict,
    Unavailable,
    Timeout,
    Internal,
}

impl ErrorCode {
    /// The exit code paired with this error class.
    pub fn exit(self) -> ExitCode {
        match self {
            ErrorCode::Usage => ExitCode::Usage,
            ErrorCode::NotFound => ExitCode::NotFound,
            ErrorCode::Permission => ExitCode::Permission,
            ErrorCode::Conflict => ExitCode::Conflict,
            ErrorCode::Unavailable
            | ErrorCode::Timeout
            | ErrorCode::Internal => ExitCode::Failure,
        }
    }
}

/// A structured CLI error.
///
/// In machine mode it renders as the §4 error envelope
/// (`{ "error": { code, exit_code, message, hint, timestamp, command,
/// docs_url } }`) on stderr; in human mode it renders as a short message plus
/// the hint.
#[derive(Debug)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    /// A runnable next command (tips-thinking). Never a sentence about a
    /// command — the actual command an agent can execute.
    pub hint: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
        }
    }

    /// Attach a runnable hint command.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Usage, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unavailable, message)
    }

    pub fn exit_code(&self) -> ExitCode {
        self.code.exit()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, "\nhint: {hint}")?;
        }
        Ok(())
    }
}

impl std::error::Error for AppError {}
