// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command::Command;
use sentinel4rust::system::unit::job_toml::JobToml;
use std::path::Path;
use std::time::Duration;

#[test]
fn parse_of_a_job_without_a_program_fails_naming_it() {
    // Arrange
    let text = "dir = \"/w\"\ntimeout_seconds = 1\n";

    // Act
    let parsed = JobToml.parse(text);

    // Assert
    assert!(parsed.is_err_and(|error| error.0.contains("program")));
}

#[test]
fn parse_of_a_rendered_command_round_trips_it() {
    // Arrange
    let command = Command::new(
        "cargo",
        &[
            "mutants",
            "--re",
            r#"^(?:src/a\.rs:1:1: replace "x" with y)$"#,
        ],
        Path::new("/w/runs/r"),
        Duration::from_secs(43_200),
    )
    .with_env("TMPDIR", "/w/tmp/r")
    .with_env("NO_COLOR", "1");

    // Act
    let parsed = JobToml.parse(&JobToml.render(&command));

    // Assert
    assert_eq!(parsed, Ok(command));
}
