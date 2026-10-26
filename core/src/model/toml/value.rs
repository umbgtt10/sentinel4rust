// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A scalar value.

use alloc::string::String;

/// A scalar value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A string.
    String(String),
    /// An integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
}
