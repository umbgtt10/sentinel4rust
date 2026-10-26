// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::traits::clock::Clock;
use sentinel4rust_cli::system::process::system_clock::SystemClock;

#[test]
fn now_on_this_machine_is_after_2026() {
    // Arrange & Act
    let now = SystemClock.now();

    // Assert
    assert!(now > Timestamp::from_unix(1_767_225_600));
}
