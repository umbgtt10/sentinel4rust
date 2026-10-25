// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Running programs.

use crate::model::command::Command;
use crate::model::command_result::CommandResult;

/// Runs programs.
pub trait CommandRunner {
    /// Runs `command` to completion or its deadline.
    fn run(&self, command: &Command) -> CommandResult;
}
