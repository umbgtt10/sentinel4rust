// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The system clock.

use std::time::{SystemTime, UNIX_EPOCH};

use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::traits::clock::Clock;

/// The system clock.
#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Timestamp::from_unix(i64::try_from(seconds).unwrap_or(i64::MAX))
    }
}
