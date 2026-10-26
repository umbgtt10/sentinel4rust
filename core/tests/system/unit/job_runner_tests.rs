// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust::system::unit::job_runner::JobRunner;
use sentinel4rust::system::unit::job_toml::JobToml;
use sentinel4rust::system::unit::result_toml::ResultToml;
use std::fs;
use std::rc::Rc;
use std::time::Duration;

#[test]
fn run_executes_the_job_and_writes_its_result_and_output() {
    // Arrange
    let job = TempDir::new("job");
    let command = Command::new(
        "sh",
        &["-c", "echo out $JOB_ENV; echo err >&2; exit 3"],
        &job.path().to_string_lossy(),
        Duration::from_secs(10),
    )
    .with_env("JOB_ENV", "set");
    fs::write(job.path().join("job.toml"), JobToml.render(&command)).unwrap();

    // Act
    let ran = JobRunner::new(Rc::new(SystemCommandRunner)).run(job.path());

    // Assert
    assert_eq!(ran, Ok(()));
    let stdout = fs::read_to_string(job.path().join("stdout")).unwrap();
    let stderr = fs::read_to_string(job.path().join("stderr")).unwrap();
    let result = fs::read_to_string(job.path().join("result.toml")).unwrap();
    assert_eq!(
        ResultToml.parse(&result, &stdout, &stderr),
        Ok(CommandResult::Exited {
            code: Some(3),
            stdout: "out set\n".to_string(),
            stderr: "err\n".to_string(),
        })
    );
}

#[test]
fn run_without_a_job_fails_naming_the_file() {
    // Arrange
    let job = TempDir::new("no-job");

    // Act
    let ran = JobRunner::new(Rc::new(SystemCommandRunner)).run(job.path());

    // Assert
    assert!(ran.is_err_and(|error| error.0.contains("job.toml")));
}
