// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::system::process::process_group::ProcessGroup;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use std::process::Stdio;

// The background child would print `survived` a second in; its stdout ends
// only when every process holding it is gone, so a survivor shows up.
#[test]
fn kill_ends_the_leader_and_everything_it_started() {
    // Arrange
    let mut leader = Command::new("sh")
        .args(["-c", "(sleep 1; echo survived) & echo started; sleep 30"])
        .stdout(Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("sh starts");
    let mut stdout = leader.stdout.take().expect("piped");
    let mut first = [0_u8; 8];
    stdout
        .read_exact(&mut first)
        .expect("the leader says it started");

    // Act
    ProcessGroup::new(leader.id()).kill();

    // Assert
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).expect("stdout ends");
    assert_eq!(String::from_utf8_lossy(&first), "started\n");
    assert_eq!(rest, "");
    assert_eq!(leader.wait().expect("reaped").signal(), Some(9));
}
