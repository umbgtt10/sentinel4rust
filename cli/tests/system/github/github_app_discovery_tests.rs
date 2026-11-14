// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fixed_token::FixedToken;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::github::discovered_repo::DiscoveredRepo;
use sentinel4rust::traits::repo_discovery::RepoDiscovery;
use sentinel4rust_cli::system::github::github_api::GithubApi;
use sentinel4rust_cli::system::github::github_app_discovery::GithubAppDiscovery;
use sentinel4rust_cli::system::github::secret_dir::SecretDir;
use std::rc::Rc;

fn discovery(runner: &Rc<ScriptedRunner>, dir: &TempDir) -> GithubAppDiscovery {
    GithubAppDiscovery::new(GithubApi::new(
        runner.clone(),
        Rc::new(FixedToken(String::from("ghs_t"))),
        SecretDir::new(dir.path().join("github")),
    ))
}

fn page(repos: &[(&str, bool)]) -> String {
    let entries: Vec<String> = repos
        .iter()
        .map(|(name, archived)| {
            format!(
                r#"{{"name":"{name}","full_name":"o/{name}","clone_url":"https://github.com/o/{name}.git","default_branch":"main","archived":{archived}}}"#
            )
        })
        .collect();
    format!(
        r#"{{"total_count":{},"repositories":[{}]}}"#,
        repos.len(),
        entries.join(",")
    )
}

#[test]
fn discover_keeps_unarchived_repositories_with_mutants_filters() {
    // Arrange
    let dir = TempDir::new("discover");
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(&page(&[("kept", false), ("old", true), ("plain", false)])),
        ok("200"),
        ok("404"),
    ]));

    // Act
    let found = discovery(&runner, &dir).discover().unwrap();

    // Assert
    assert_eq!(
        found,
        [DiscoveredRepo {
            name: String::from("kept"),
            url: String::from("https://github.com/o/kept.git"),
            branch: String::from("main"),
        }]
    );
    let urls: Vec<String> = runner
        .commands()
        .iter()
        .map(|command| command.args().last().unwrap().clone())
        .collect();
    assert_eq!(
        urls,
        [
            "https://api.github.com/installation/repositories?per_page=100&page=1",
            "https://api.github.com/repos/o/kept/contents/.cargo/mutants.toml?ref=main",
            "https://api.github.com/repos/o/plain/contents/.cargo/mutants.toml?ref=main"
        ]
    );
}

#[test]
fn discover_of_an_unreadable_answer_fails() {
    // Arrange
    let dir = TempDir::new("discover-bad");
    let runner = Rc::new(ScriptedRunner::new(vec![ok("<html>")]));

    // Act
    let error = discovery(&runner, &dir).discover().unwrap_err();

    // Assert
    assert!(error.0.contains("installation/repositories"), "{error}");
}

#[test]
fn discover_reads_every_page_until_a_short_one() {
    // Arrange
    let dir = TempDir::new("discover-pages");
    let names: Vec<String> = (0..100).map(|i| format!("old{i}")).collect();
    let full: Vec<(&str, bool)> = names.iter().map(|name| (name.as_str(), true)).collect();
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(&page(&full)),
        ok(&page(&[("last", false)])),
        ok("200"),
    ]));

    // Act
    let found = discovery(&runner, &dir).discover().unwrap();

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(
        runner.commands()[1].args().last().unwrap(),
        "https://api.github.com/installation/repositories?per_page=100&page=2"
    );
}
