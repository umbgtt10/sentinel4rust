// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! git, run through a `CommandRunner`.

use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use crate::model::command::Command;
use crate::model::command_result::CommandResult;
use crate::system::git::git_constants::LOCAL_TIMEOUT;
use crate::system::git::git_failure::GitFailure;
use crate::traits::command_runner::CommandRunner;

/// git, run through a [`CommandRunner`], never prompting and always in the C locale.
#[derive(Clone)]
pub struct GitRunner {
    runner: Rc<dyn CommandRunner>,
}

impl GitRunner {
    /// git run by `runner`.
    pub fn new(runner: Rc<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    /// The git command line `args` in `dir`, killed after `timeout`.
    pub fn command(&self, dir: &Path, args: &[&str], timeout: Duration) -> Command {
        Command::new("git", args, &dir.to_string_lossy(), timeout)
            .with_env("GIT_TERMINAL_PROMPT", "0")
            .with_env("LC_ALL", "C")
    }

    /// Runs git with `args` in `dir`; its stdout, or how it failed.
    pub fn git(
        &self,
        dir: &Path,
        args: &[&str],
        timeout: Duration,
    ) -> Result<String, CommandResult> {
        self.run(&self.command(dir, args, timeout))
    }

    /// Runs a git command that does not talk to the remote; a failure names the command line.
    pub fn local(&self, dir: &Path, args: &[&str]) -> Result<String, GitFailure> {
        self.git(dir, args, LOCAL_TIMEOUT)
            .map_err(|result| GitFailure {
                what: format!("git {}", args.join(" ")),
                result,
            })
    }

    /// Runs `command`; its stdout, or how it failed.
    pub fn run(&self, command: &Command) -> Result<String, CommandResult> {
        let result = self.runner.run(command);
        match result.success() {
            Some(stdout) => Ok(stdout.to_string()),
            None => Err(result),
        }
    }
}
