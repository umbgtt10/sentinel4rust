// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::invocation::command_line::CommandLine;
use sentinel4rust::invocation::usage_error::UsageError;
use std::path::PathBuf;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

#[test]
fn parse_bad_command_lines_fails() {
    // Arrange & Act & Assert
    assert!(CommandLine::parse(&args(&[])).is_err());
    assert!(CommandLine::parse(&args(&["--roster"])).is_err());
    assert!(CommandLine::parse(&args(&["--roster", "r", "--frobnicate"])).is_err());
}

#[test]
fn parse_bad_command_lines_names_each_problem() {
    // Arrange & Act & Assert
    assert_eq!(
        CommandLine::parse(&args(&[])),
        Err(UsageError::MissingRoster)
    );
    assert_eq!(
        CommandLine::parse(&args(&["--roster"])),
        Err(UsageError::MissingRosterFile)
    );
    assert_eq!(
        CommandLine::parse(&args(&["--roster", "r", "--frobnicate"])),
        Err(UsageError::UnknownArgument(String::from("--frobnicate")))
    );
}

#[test]
fn parse_valid_forms_reads_each_invocation() {
    // Arrange & Act & Assert
    assert_eq!(
        CommandLine::parse(&args(&["--roster", "r.toml", "--once"])),
        Ok(CommandLine::Run {
            roster: PathBuf::from("r.toml"),
            once: true
        })
    );
    assert_eq!(
        CommandLine::parse(&args(&["--roster=r.toml"])),
        Ok(CommandLine::Run {
            roster: PathBuf::from("r.toml"),
            once: false
        })
    );
    assert_eq!(
        CommandLine::parse(&args(&["--help"])),
        Ok(CommandLine::Help)
    );
    assert_eq!(CommandLine::parse(&args(&["-V"])), Ok(CommandLine::Version));
}
