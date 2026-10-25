// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust::traits::command_runner::CommandRunner;

// The system runner, letting git clone submodules over `file://` -- which git
// refuses by default since 2.38 -- so a test can give a repository a local
// submodule. The service adds nothing of the kind.
pub struct FileProtocolRunner;

impl CommandRunner for FileProtocolRunner {
    fn run(&self, command: &Command) -> CommandResult {
        SystemCommandRunner.run(&command.clone().with_env("GIT_ALLOW_PROTOCOL", "file"))
    }
}
