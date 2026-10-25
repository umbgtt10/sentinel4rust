// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust::system::unit::job_runner::JobRunner;
use sentinel4rust::system::unit::unit_command_runner::UnitCommandRunner;
use sentinel4rust::traits::command_runner::CommandRunner;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

// systemd's stand-in: asked to start the unit, it runs the job the way the
// unit's `sentinel4rust --run-job` would, then answers as systemctl does.
struct FakeSystemd {
    started: RefCell<Vec<Command>>,
    refuse: bool,
}

impl CommandRunner for FakeSystemd {
    fn run(&self, command: &Command) -> CommandResult {
        self.started.borrow_mut().push(command.clone());
        if self.refuse {
            return CommandResult::Exited {
                code: Some(1),
                stdout: String::new(),
                stderr: "Failed to start sentinel4rust-mutants@run.service: Access denied"
                    .to_string(),
            };
        }
        let ran = JobRunner::new(Rc::new(SystemCommandRunner)).run(command.dir());
        CommandResult::Exited {
            code: Some(i32::from(ran.is_err())),
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

fn runner(jobs: &Path, refuse: bool) -> (UnitCommandRunner, Rc<FakeSystemd>) {
    let systemd = Rc::new(FakeSystemd {
        started: RefCell::new(Vec::new()),
        refuse,
    });
    let runner = UnitCommandRunner::new(
        systemd.clone(),
        jobs.to_path_buf(),
        "sentinel4rust-mutants".to_string(),
    );
    (runner, systemd)
}

#[test]
fn run_starts_the_unit_and_answers_with_what_the_job_did() {
    // Arrange
    let jobs = TempDir::new("jobs");
    let (runner, systemd) = runner(jobs.path(), false);
    let command = Command::new(
        "sh",
        &["-c", "echo in the unit; exit 2"],
        jobs.path(),
        Duration::from_secs(10),
    );

    // Act
    let result = runner.run(&command);

    // Assert
    assert_eq!(
        result,
        CommandResult::Exited {
            code: Some(2),
            stdout: "in the unit\n".to_string(),
            stderr: String::new(),
        }
    );
    let started = systemd.started.borrow();
    assert_eq!(
        started[0].args(),
        ["start", "--wait", "sentinel4rust-mutants@run.service"]
    );
    assert_eq!(started[0].program(), "systemctl");
    assert_eq!(started[0].dir(), jobs.path().join("run"));
    assert!(started[0].timeout() > Duration::from_secs(10));
}

#[test]
fn run_with_the_unit_refused_fails_to_start_naming_systemctl() {
    // Arrange
    let jobs = TempDir::new("jobs-refused");
    let (runner, _) = runner(jobs.path(), true);
    let command = Command::new("true", &[], jobs.path(), Duration::from_secs(10));

    // Act
    let result = runner.run(&command);

    // Assert
    assert_eq!(
        result,
        CommandResult::FailedToStart(
            "systemctl start --wait sentinel4rust-mutants@run.service: exited with 1: Failed to start sentinel4rust-mutants@run.service: Access denied"
                .to_string()
        )
    );
}
