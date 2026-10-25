// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What to do next.

use crate::model::timestamp::Timestamp;

/// What to do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// Visit the repository at this roster index.
    Visit(usize),
    /// Nothing is due before this instant.
    SleepUntil(Timestamp),
}
