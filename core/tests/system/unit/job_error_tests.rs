// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::toml_error::TomlError;
use sentinel4rust::system::unit::job_error::JobError;

#[test]
fn from_a_toml_error_keeps_its_line_and_message() {
    // Arrange
    let error = TomlError {
        line: 2,
        message: String::from("expected `=`"),
    };

    // Act
    let job_error = JobError::from(error);

    // Assert
    assert_eq!(job_error, JobError(String::from("line 2: expected `=`")));
}

#[test]
fn to_string_names_the_job_and_the_problem() {
    // Arrange & Act
    let text = JobError(String::from("read /w/jobs/run/job.toml: not found")).to_string();

    // Assert
    assert_eq!(text, "job: read /w/jobs/run/job.toml: not found");
}
