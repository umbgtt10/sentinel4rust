// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::discovered_repo::DiscoveredRepo;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::onboarding::toml_roster::TomlRoster;
use sentinel4rust::patrol::watch_list::WatchList;
use std::time::Duration;

fn discovered(name: &str) -> DiscoveredRepo {
    DiscoveredRepo {
        name: name.to_string(),
        url: format!("https://github.com/o/{name}.git"),
        branch: String::from("main"),
    }
}

fn names(repos: &[OnboardedRepo]) -> Vec<&str> {
    repos.iter().map(|repo| repo.name.as_str()).collect()
}

fn watch_list(repos: &str) -> WatchList {
    let roster = TomlRoster
        .parse(&format!(
            "work_dir = \"/w\"\nresults_url = \"git@github-mutants:o/mutants.git\"\njobs = 3\nrun_timeout_hours = 2\ngithub_app_id = 7\ngithub_app_installation_id = 8\ngithub_app_key = \"/k\"\n{repos}"
        ))
        .unwrap();
    WatchList::new(&roster)
}

#[test]
fn merge_adds_discovered_repositories_sorted_with_the_defaults() {
    // Arrange
    let list = watch_list("");

    // Act
    let repos = list.merge(&[discovered("zeta"), discovered("alpha")]);

    // Assert
    assert_eq!(names(&repos), ["alpha", "zeta"]);
    assert_eq!(repos[0].url, "https://github.com/o/alpha.git");
    assert_eq!(repos[0].branch, "main");
    assert_eq!(repos[0].jobs, 3);
    assert_eq!(repos[0].run_timeout, Duration::from_secs(7200));
}

#[test]
fn merge_keeps_an_explicit_entry_first_and_over_a_discovered_one() {
    // Arrange
    let list = watch_list(
        "[[repo]]\nname = \"zeta\"\nurl = \"file:///z.git\"\njobs = 1\n[[repo]]\nname = \"elsewhere\"\nurl = \"file:///e.git\"\n",
    );

    // Act
    let repos = list.merge(&[discovered("zeta"), discovered("alpha")]);

    // Assert
    assert_eq!(names(&repos), ["zeta", "elsewhere", "alpha"]);
    assert_eq!(repos[0].url, "file:///z.git");
    assert_eq!(repos[0].jobs, 1);
}

#[test]
fn merge_leaves_out_the_results_repository_and_unusable_names() {
    // Arrange
    let list = watch_list("");

    // Act
    let repos = list.merge(&[
        discovered("mutants"),
        discovered(".github"),
        discovered("ok"),
    ]);

    // Assert
    assert_eq!(names(&repos), ["ok"]);
}

#[test]
fn merge_of_nothing_discovered_is_the_explicit_entries() {
    // Arrange
    let list = watch_list("[[repo]]\nname = \"a\"\nurl = \"u\"\n");

    // Act
    let repos = list.merge(&[]);

    // Assert
    assert_eq!(names(&repos), ["a"]);
}
