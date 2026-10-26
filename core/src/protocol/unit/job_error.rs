// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A job that could not be read, run or recorded.

use alloc::string::String;
use alloc::string::ToString;
use core::fmt;

use crate::model::toml::toml_error::TomlError;

/// A job that could not be read, run or recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobError(pub String);

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "job: {}", self.0)
    }
}

impl From<TomlError> for JobError {
    fn from(error: TomlError) -> Self {
        Self(error.to_string())
    }
}
