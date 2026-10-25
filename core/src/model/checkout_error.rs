// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why a checkout could not be made.

use std::fmt;

/// Why a checkout could not be made: the step that failed and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutError(pub String);

impl fmt::Display for CheckoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
