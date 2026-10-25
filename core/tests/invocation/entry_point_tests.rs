// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::invocation::entry_point::EntryPoint;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::onboarding::roster::Roster;
use sentinel4rust::model::round_step::RoundStep;
use std::fs;
use std::process::ExitCode;
use std::time::Duration;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

// A roster whose one repository does not exist: real git says so at once.
fn unreachable_roster(dir: &TempDir) -> Roster {
    Roster {
        work_dir: dir.path().join("work"),
        pause: Duration::from_secs(300),
        jobs: 1,
        run_timeout: Duration::from_secs(60),
        config_version: 1,
        repos: vec![OnboardedRepo {
            name: String::from("x"),
            url: format!("file://{}/nowhere.git", dir.path().display()),
            branch: String::from("main"),
            jobs: 1,
        }],
    }
}

#[test]
fn compose_of_a_roster_wires_the_real_git() {
    // Arrange
    let dir = TempDir::new("compose");
    let roster = unreachable_roster(&dir);

    // Act
    let steps = EntryPoint.compose(roster).run_once();

    // Assert
    assert!(
        matches!(&steps[..], [RoundStep::Unreachable { reason }] if reason.contains("git ls-remote")),
        "{steps:?}"
    );
    assert!(dir.path().join("work/repos/x.git/HEAD").is_file());
}

#[test]
fn run_bad_command_line_exits_two() {
    // Arrange & Act
    let code = EntryPoint.run(&args(&["--frobnicate"]));

    // Assert
    assert_eq!(code, ExitCode::from(2));
}

#[test]
fn run_help_succeeds() {
    // Arrange & Act
    let code = EntryPoint.run(&args(&["--help"]));

    // Assert
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn run_once_with_an_unreachable_repository_exits_one() {
    // Arrange
    let dir = TempDir::new("once");
    let roster = dir.path().join("r.toml");
    fs::write(
        &roster,
        format!(
            "work_dir = \"{}/work\"\n[[repo]]\nname = \"x\"\nurl = \"file://{}/nowhere.git\"\n",
            dir.path().display(),
            dir.path().display()
        ),
    )
    .unwrap();

    // Act
    let code = EntryPoint.run(&args(&["--roster", &roster.to_string_lossy(), "--once"]));

    // Assert
    assert_eq!(code, ExitCode::from(1));
}

#[test]
fn run_unreadable_roster_fails() {
    // Arrange
    let dir = TempDir::new("unreadable");
    let roster = dir.path().join("missing.toml");

    // Act
    let code = EntryPoint.run(&args(&["--roster", &roster.to_string_lossy()]));

    // Assert
    assert_eq!(code, ExitCode::FAILURE);
}

#[test]
fn run_version_succeeds() {
    // Arrange & Act
    let code = EntryPoint.run(&args(&["--version"]));

    // Assert
    assert_eq!(code, ExitCode::SUCCESS);
}
