// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::traits::command_runner::CommandRunner;
use std::cell::RefCell;
use std::collections::VecDeque;

pub fn failed(code: i32, stderr: &str) -> CommandResult {
    CommandResult::Exited {
        code: Some(code),
        stdout: String::new(),
        stderr: String::from(stderr),
    }
}

pub fn ok(stdout: &str) -> CommandResult {
    CommandResult::Exited {
        code: Some(0),
        stdout: String::from(stdout),
        stderr: String::new(),
    }
}

// A command runner answering from a script, in order, and keeping every
// command it was given. A command beyond the script is a test bug and panics.
pub struct ScriptedRunner {
    replies: RefCell<VecDeque<CommandResult>>,
    commands: RefCell<Vec<Command>>,
}

impl ScriptedRunner {
    pub fn new(replies: Vec<CommandResult>) -> Self {
        Self {
            replies: RefCell::new(replies.into()),
            commands: RefCell::new(Vec::new()),
        }
    }

    pub fn args(&self) -> Vec<Vec<String>> {
        self.commands
            .borrow()
            .iter()
            .map(|command| command.args().to_vec())
            .collect()
    }

    pub fn commands(&self) -> Vec<Command> {
        self.commands.borrow().clone()
    }
}

impl CommandRunner for ScriptedRunner {
    fn run(&self, command: &Command) -> CommandResult {
        self.commands.borrow_mut().push(command.clone());
        self.replies
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| panic!("unscripted command: {}", command.display()))
    }
}
