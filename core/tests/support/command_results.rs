// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command_result::CommandResult;

pub fn failed(code: i32, stderr: &str) -> CommandResult {
    CommandResult::Exited {
        code: Some(code),
        stdout: String::new(),
        stderr: String::from(stderr),
    }
}
