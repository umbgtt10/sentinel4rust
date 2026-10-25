// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::system::git::crate_readiness::CrateReadiness;
use sentinel4rust::system::git::git_runner::GitRunner;
use std::path::Path;
use std::rc::Rc;

fn head() -> CommitHash {
    CommitHash::parse(&"ab".repeat(20)).unwrap()
}

#[test]
fn check_with_cargo_toml_and_mutants_config_is_ready() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok(""), ok("")]));
    let readiness = CrateReadiness::new(GitRunner::new(runner.clone()));

    // Act
    let result = readiness.check(Path::new("/w/repos/x.git"), &head());

    // Assert
    assert_eq!(result, Ok(()));
    let at_head = format!("{}:", head());
    assert_eq!(
        runner.args(),
        [
            ["cat-file", "-e", &format!("{at_head}Cargo.toml")],
            ["cat-file", "-e", &format!("{at_head}.cargo/mutants.toml")],
        ]
    );
}

#[test]
fn check_without_cargo_toml_is_not_a_crate() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![failed(128, "")]));
    let readiness = CrateReadiness::new(GitRunner::new(runner.clone()));

    // Act
    let result = readiness.check(Path::new("/w/repos/x.git"), &head());

    // Assert
    assert_eq!(result, Err(SkipReason::NotACrate));
    assert_eq!(runner.commands().len(), 1);
}

#[test]
fn check_without_mutants_config_has_no_filters() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok(""), failed(128, "")]));
    let readiness = CrateReadiness::new(GitRunner::new(runner));

    // Act
    let result = readiness.check(Path::new("/w/repos/x.git"), &head());

    // Assert
    assert_eq!(result, Err(SkipReason::NoFilters));
}
