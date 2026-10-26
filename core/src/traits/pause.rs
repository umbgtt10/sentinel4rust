// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Waiting.

use core::time::Duration;

/// Waiting.
pub trait Pause {
    /// Waits for `duration`.
    fn pause(&self, duration: Duration);
}
