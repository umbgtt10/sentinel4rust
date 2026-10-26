// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A roster that cannot be used.

use alloc::string::String;
use alloc::string::ToString;
use core::error::Error;
use core::fmt;

use crate::model::toml::toml_error::TomlError;

/// A roster that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterError(pub String);

impl fmt::Display for RosterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "roster: {}", self.0)
    }
}

impl Error for RosterError {}

impl From<TomlError> for RosterError {
    fn from(error: TomlError) -> Self {
        Self(error.to_string())
    }
}
