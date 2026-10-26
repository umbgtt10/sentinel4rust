// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::roster_error::RosterError;
use sentinel4rust::system::unit::job_error::JobError;
use sentinel4rust_cli::invocation::startup_error::StartupError;
use std::path::PathBuf;

#[test]
fn from_a_job_error_is_a_job_failure_worded_as_the_job() {
    // Arrange
    let error = JobError(String::from("read /w/jobs/run/job.toml: not found"));

    // Act
    let startup = StartupError::from(error.clone());

    // Assert
    assert_eq!(startup, StartupError::Job(error));
    assert_eq!(
        startup.to_string(),
        "job: read /w/jobs/run/job.toml: not found"
    );
}

#[test]
fn from_a_roster_error_is_invalid() {
    // Arrange
    let error = RosterError(String::from("no [[repo]] entries"));

    // Act
    let startup = StartupError::from(error.clone());

    // Assert
    assert_eq!(startup, StartupError::Invalid(error));
}

#[test]
fn to_string_of_each_reads_as_the_journal_always_said_it() {
    // Arrange
    let errors = [
        StartupError::Unreadable {
            path: PathBuf::from("/etc/r.toml"),
            reason: String::from("No such file or directory (os error 2)"),
        },
        StartupError::Invalid(RosterError(String::from("no [[repo]] entries"))),
        StartupError::NoWorkDir {
            path: PathBuf::from("/srv/s"),
            reason: String::from("Permission denied (os error 13)"),
        },
        StartupError::NoLockFile {
            path: PathBuf::from("/srv/s/sentinel4rust.lock"),
            reason: String::from("Permission denied (os error 13)"),
        },
        StartupError::Locked {
            work_dir: PathBuf::from("/srv/s"),
            lock: PathBuf::from("/srv/s/sentinel4rust.lock"),
        },
    ];

    // Act
    let texts: Vec<String> = errors.iter().map(ToString::to_string).collect();

    // Assert
    assert_eq!(
        texts,
        [
            "read /etc/r.toml: No such file or directory (os error 2)",
            "roster: no [[repo]] entries",
            "create /srv/s: Permission denied (os error 13)",
            "open /srv/s/sentinel4rust.lock: Permission denied (os error 13)",
            "another sentinel4rust is running on /srv/s (lock /srv/s/sentinel4rust.lock)",
        ]
    );
}
