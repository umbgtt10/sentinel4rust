// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why a `summary.toml` could not be used.

use alloc::string::String;
use alloc::string::ToString;
use core::fmt;

use crate::model::toml::toml_error::TomlError;

/// Why a `summary.toml` could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// Written by a newer version of the tool; treated as no record, so the commit is run again.
    UnknownSchema(i64),
    /// Not valid.
    Invalid(String),
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSchema(schema) => write!(f, "unknown summary schema {schema}"),
            Self::Invalid(message) => write!(f, "invalid summary: {message}"),
        }
    }
}

impl From<TomlError> for RecordError {
    fn from(error: TomlError) -> Self {
        Self::Invalid(error.to_string())
    }
}
