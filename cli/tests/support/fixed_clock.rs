// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::traits::clock::Clock;
use std::cell::Cell;

// A clock that moves only when a test moves it.
pub struct FixedClock(pub Cell<i64>);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_unix(self.0.get())
    }
}
