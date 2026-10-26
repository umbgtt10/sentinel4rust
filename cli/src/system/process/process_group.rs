// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A process group, killed as one.

use std::process::{Command, Stdio};

/// The process group a child leads (it was started with `process_group(0)`, so its pid is the group id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessGroup {
    leader: u32,
}

impl ProcessGroup {
    /// The group led by `leader`.
    pub fn new(leader: u32) -> Self {
        Self { leader }
    }

    /// Sends `SIGKILL` to every process in the group.
    pub fn kill(&self) {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", self.leader)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
