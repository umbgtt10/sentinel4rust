// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The current time.

use crate::model::timestamp::Timestamp;

/// The current time.
pub trait Clock {
    /// Now, UTC.
    fn now(&self) -> Timestamp;
}
