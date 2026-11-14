// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fixed_clock::FixedClock;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::command::Command;
use sentinel4rust::model::onboarding::github_app::GithubApp;
use sentinel4rust_cli::invocation::github_access::GithubAccess;
use std::cell::Cell;
use std::fs;
use std::rc::Rc;
use std::time::Duration;

const TOKEN_REPLY: &str = r#"{"token":"ghs_m","expires_at":"2027-01-15T09:00:00Z"}"#;

fn access(dir: &TempDir, runner: &Rc<ScriptedRunner>) -> GithubAccess {
    let secrets = dir.path().join("sentinel4rust-github");
    fs::create_dir_all(&secrets).unwrap();
    fs::write(secrets.join("jwt.sig"), [1, 2, 3]).unwrap();
    GithubAccess::new(
        runner.clone(),
        Rc::new(FixedClock(Cell::new(1_800_000_000))),
        &GithubApp {
            app_id: 7,
            installation_id: 8,
            key: dir.path().join("app.pem").to_string_lossy().into_owned(),
        },
    )
}

#[test]
fn discovery_lists_the_installation_with_a_token_kept_beside_the_key() {
    // Arrange
    let dir = TempDir::new("access-discover");
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(""),
        ok(TOKEN_REPLY),
        ok(r#"{"total_count":0,"repositories":[]}"#),
    ]));
    let access = access(&dir, &runner);

    // Act
    let found = access.discovery().discover().unwrap();

    // Assert
    assert_eq!(found.len(), 0);
    assert_eq!(
        runner.commands()[2].args().last().unwrap(),
        "https://api.github.com/installation/repositories?per_page=100&page=1"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("sentinel4rust-github/api.header")).unwrap(),
        "Authorization: Bearer ghs_m\n"
    );
}

#[test]
fn git_runner_gives_git_the_minted_token() {
    // Arrange
    let dir = TempDir::new("access-git");
    let runner = Rc::new(ScriptedRunner::new(vec![ok(""), ok(TOKEN_REPLY), ok("")]));
    let access = access(&dir, &runner);
    let fetch = Command::new("git", &["fetch", "origin"], "/w", Duration::from_secs(60));

    // Act
    access.git_runner().run(&fetch);

    // Assert
    assert_eq!(runner.commands()[0].program(), "openssl");
    assert_eq!(
        runner.commands()[2].env()["GIT_CONFIG_VALUE_0"],
        "AUTHORIZATION: basic eC1hY2Nlc3MtdG9rZW46Z2hzX20="
    );
}
