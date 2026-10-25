// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::system::unit::result_toml::ResultToml;

#[test]
fn parse_of_an_unknown_kind_fails() {
    // Arrange & Act
    let parsed = ResultToml.parse("kind = \"vanished\"\n", "", "");

    // Assert
    assert!(parsed.is_err_and(|error| error.0.contains("vanished")));
}

#[test]
fn parse_of_each_rendered_result_round_trips_it_with_its_output() {
    // Arrange
    let results = [
        CommandResult::Exited {
            code: Some(3),
            stdout: "out\n".to_string(),
            stderr: "err\n".to_string(),
        },
        CommandResult::Exited {
            code: None,
            stdout: String::new(),
            stderr: "killed".to_string(),
        },
        CommandResult::TimedOut {
            stdout: "partial".to_string(),
            stderr: String::new(),
        },
        CommandResult::FailedToStart("No such file or directory (os error 2)".to_string()),
    ];

    // Act
    let parsed: Vec<CommandResult> = results
        .iter()
        .map(|result| {
            let (stdout, stderr) = match result {
                CommandResult::Exited { stdout, stderr, .. }
                | CommandResult::TimedOut { stdout, stderr } => (stdout.as_str(), stderr.as_str()),
                CommandResult::FailedToStart(_) => ("", ""),
            };
            ResultToml
                .parse(&ResultToml.render(result), stdout, stderr)
                .unwrap()
        })
        .collect();

    // Assert
    assert_eq!(parsed, results);
}
