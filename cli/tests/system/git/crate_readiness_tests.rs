// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust_cli::system::git::crate_readiness::CrateReadiness;
use sentinel4rust_cli::system::git::git_runner::GitRunner;
use std::path::Path;
use std::rc::Rc;

fn head() -> CommitHash {
    CommitHash::parse(&"ab".repeat(20)).unwrap()
}

#[test]
fn check_with_a_path_dependency_outside_is_skipped_naming_its_manifest() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(""),
        ok(""),
        ok("Cargo.toml\ncrates/a/Cargo.toml\ncrates/a/src/lib.rs\n"),
        ok("[workspace]\nmembers = [\"crates/a\"]\n"),
        ok("[dependencies]\nsibling = { path = \"../../../sibling\" }\n"),
    ]));
    let readiness = CrateReadiness::new(GitRunner::new(runner.clone()));

    // Act
    let result = readiness.check(Path::new("/w/repos/x.git"), &head());

    // Assert
    assert_eq!(
        result,
        Err(SkipReason::PathOutsideRepo {
            manifest: String::from("crates/a/Cargo.toml")
        })
    );
    let at_head = format!("{}:", head());
    assert_eq!(
        runner.args()[4],
        ["show", &format!("{at_head}crates/a/Cargo.toml")]
    );
}

#[test]
fn check_with_an_unlistable_tree_is_ready() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok(""), ok(""), failed(128, "")]));
    let readiness = CrateReadiness::new(GitRunner::new(runner));

    // Act
    let result = readiness.check(Path::new("/w/repos/x.git"), &head());

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn check_with_cargo_toml_and_mutants_config_is_ready() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(""),
        ok(""),
        ok("Cargo.toml\nsrc/lib.rs\n"),
        ok("[package]\nname = \"x\"\n"),
    ]));
    let readiness = CrateReadiness::new(GitRunner::new(runner.clone()));

    // Act
    let result = readiness.check(Path::new("/w/repos/x.git"), &head());

    // Assert
    assert_eq!(result, Ok(()));
    let at_head = format!("{}:", head());
    assert_eq!(
        runner.args(),
        [
            vec!["cat-file", "-e", &format!("{at_head}Cargo.toml")],
            vec!["cat-file", "-e", &format!("{at_head}.cargo/mutants.toml")],
            vec!["ls-tree", "-r", "--name-only", head().as_str()],
            vec!["show", &format!("{at_head}Cargo.toml")],
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
