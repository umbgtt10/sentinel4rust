// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Sleeping the current thread.

use std::thread;
use std::time::Duration;

use sentinel4rust::traits::pause::Pause;

/// Sleeps the current thread.
#[derive(Debug, Default)]
pub struct ThreadPause;

impl Pause for ThreadPause {
    fn pause(&self, duration: Duration) {
        thread::sleep(duration);
    }
}
