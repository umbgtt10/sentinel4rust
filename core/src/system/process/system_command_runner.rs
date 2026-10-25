// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Running commands with `std::process`.

use std::os::unix::process::CommandExt;
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};

use crate::model::command::Command;
use crate::model::command_result::CommandResult;
use crate::system::process::process_group::ProcessGroup;
use crate::system::process::stream_reader::StreamReader;
use crate::traits::command_runner::CommandRunner;

const POLL: Duration = Duration::from_millis(20);

/// Runs commands with `std::process`, each in its own process group so a timeout kills its children too.
#[derive(Debug, Default)]
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, command: &Command) -> CommandResult {
        let spawned = ProcessCommand::new(command.program())
            .args(command.args())
            .current_dir(command.dir())
            .envs(command.env())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => return CommandResult::FailedToStart(error.to_string()),
        };
        let group = ProcessGroup::new(child.id());
        let stdout = StreamReader.spawn(child.stdout.take());
        let stderr = StreamReader.spawn(child.stderr.take());
        let deadline = Instant::now() + command.timeout();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() >= deadline => {
                    group.kill();
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                Ok(None) => thread::sleep(POLL),
                Err(error) => {
                    group.kill();
                    let _ = child.kill();
                    let _ = child.wait();
                    return CommandResult::FailedToStart(error.to_string());
                }
            }
        };
        let stdout = stdout.join().unwrap_or_default();
        let stderr = stderr.join().unwrap_or_default();
        match status {
            Some(status) => CommandResult::Exited {
                code: status.code(),
                stdout,
                stderr,
            },
            None => CommandResult::TimedOut { stdout, stderr },
        }
    }
}
