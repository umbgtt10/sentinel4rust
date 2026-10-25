// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::system::process::system_clock::SystemClock;
use sentinel4rust::system::process::thread_pause::ThreadPause;
use sentinel4rust::traits::clock::Clock;
use sentinel4rust::traits::pause::Pause;
use std::time::Duration;

// Read through the clock adapter, in whole seconds: a pause of one second
// moves it on by at least one, wherever in a second it starts.
#[test]
fn pause_one_second_moves_the_clock_on_by_a_second() {
    // Arrange
    let before = SystemClock.now();

    // Act
    ThreadPause.pause(Duration::from_secs(1));

    // Assert
    assert!(SystemClock.now().since(before) >= Duration::from_secs(1));
}
