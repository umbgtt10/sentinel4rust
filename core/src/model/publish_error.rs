// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why a publication did not reach `mutants`.

use std::fmt;

/// Why a publication did not reach `mutants`: the step that failed and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishError(pub String);

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
