// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why discovery found nothing to go on.

use alloc::string::String;
use core::fmt;

/// Why discovery failed: the step that failed and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryError(pub String);

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
