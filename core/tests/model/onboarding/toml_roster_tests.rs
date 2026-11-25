// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::github_app::GithubApp;
use sentinel4rust::model::onboarding::toml_roster::TomlRoster;
use std::time::Duration;

const FULL: &str = r#"
    work_dir = "/srv/s"
    results_url = "git@github-mutants:umbgtt10/mutants.git"
    pause_seconds = 60
    jobs = 6
    run_timeout_hours = 2
    config_version = 3

    [[repo]]
    name = "twin4rust"
    url = "git@github-twin4rust:umbgtt10/twin4rust.git"

    [[repo]]
    name = "etheram-raft"
    url = "git@github-etheram-raft:umbgtt10/etheram-raft.git"
    branch = "trunk"
    jobs = 2
"#;

#[test]
fn parse_full_roster_reads_settings_and_repos_in_order() {
    // Arrange & Act
    let roster = TomlRoster.parse(FULL).unwrap();

    // Assert
    assert_eq!(roster.work_dir, "/srv/s");
    assert_eq!(
        roster.results_url,
        "git@github-mutants:umbgtt10/mutants.git"
    );
    assert_eq!(roster.pause, Duration::from_secs(60));
    assert_eq!(roster.run_timeout, Duration::from_secs(7200));
    assert_eq!(roster.config_version, 3);
    let names: Vec<&str> = roster.repos.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["twin4rust", "etheram-raft"]);
    assert_eq!(roster.repos[0].branch, "main");
    assert_eq!(roster.repos[0].jobs, 6);
    assert_eq!(roster.repos[1].branch, "trunk");
    assert_eq!(roster.repos[1].jobs, 2);
}

#[test]
fn parse_github_app_reads_its_keys_and_needs_no_repo() {
    // Arrange
    let text = "\
        work_dir = \"/w\"
        results_url = \"r\"
        github_app_id = 7
        github_app_installation_id = 8
        github_app_key = \"/home/sentinel/.ssh/app.pem\"
        discovery_minutes = 30
    ";

    // Act
    let roster = TomlRoster.parse(text).unwrap();

    // Assert
    assert_eq!(
        roster.github_app,
        Some(GithubApp {
            app_id: 7,
            installation_id: 8,
            key: String::from("/home/sentinel/.ssh/app.pem"),
        })
    );
    assert_eq!(roster.discovery_interval, Duration::from_mins(30));
    assert_eq!(roster.repos.len(), 0);
}

#[test]
fn parse_invalid_rosters_fails_naming_the_problem() {
    // Arrange
    let repo = "[[repo]]\nname = \"a\"\nurl = \"u\"\n";
    let cases = [
        (repo.to_string(), "work_dir"),
        (
            "work_dir = \"/w\"\nresults_url = \"r\"\ngithub_app_id = 7".to_string(),
            "github_app_installation_id",
        ),
        (
            format!("work_dir = \"/w\"\nresults_url = \"r\"\ndiscovery_minutes = 0\n{repo}"),
            "positive",
        ),
        (format!("work_dir = \"/w\"\n{repo}"), "results_url"),
        (
            format!("work_dir = \"/w\"\nresults_url = \" \"\n{repo}"),
            "empty `results_url`",
        ),
        (
            "work_dir = \"/w\"\nresults_url = \"r\"".to_string(),
            "no [[repo]]",
        ),
        (
            format!("work_dir = \"/w\"\nresults_url = \"r\"\ntypo = 1\n{repo}"),
            "unknown key",
        ),
        (
            format!("work_dir = \"/w\"\nresults_url = \"r\"\n{repo}{repo}"),
            "duplicate name",
        ),
        (
            format!("work_dir = \"/w\"\nresults_url = \"r\"\njobs = 0\n{repo}"),
            "positive",
        ),
        (
            format!("work_dir = \"/w\"\nresults_url = \"r\"\n{repo}branch = \"mutants\""),
            "invalid branch",
        ),
        (
            "work_dir = \"/w\"\nresults_url = \"r\"\n[[repo]]\nname = \"../x\"\nurl = \"u\""
                .to_string(),
            "invalid name",
        ),
        (
            "work_dir = \"/w\"\nresults_url = \"r\"\n[[repo]]\nname = \"a\"\nurl = \" \""
                .to_string(),
            "empty url",
        ),
        (
            format!("work_dir = \"/w\"\nresults_url = \"r\"\n{repo}[[other]]\nx = 1"),
            "unknown section",
        ),
    ];

    // Act & Assert
    for (text, fragment) in cases {
        let err = TomlRoster.parse(&text).unwrap_err().0;
        assert!(err.contains(fragment), "{fragment}: {err}");
    }
}

#[test]
fn parse_minimal_roster_applies_the_defaults() {
    // Arrange & Act
    let roster = TomlRoster
        .parse("work_dir = \"/w\"\nresults_url = \"r\"\n[[repo]]\nname = \"a\"\nurl = \"u\"")
        .unwrap();

    // Assert
    assert_eq!(roster.pause, Duration::from_secs(300));
    assert_eq!(roster.jobs, 1);
    assert_eq!(roster.run_timeout, Duration::from_hours(12));
    assert_eq!(roster.config_version, 1);
    assert_eq!(roster.github_app, None);
    assert_eq!(roster.discovery_interval, Duration::from_secs(3600));
}

#[test]
fn parse_repo_run_timeout_overrides_the_global_cap() {
    // Arrange
    let text = "\
        work_dir = \"/w\"\nresults_url = \"r\"
        run_timeout_hours = 2

        [[repo]]
        name = \"small\"
        url = \"u\"

        [[repo]]
        name = \"large\"
        url = \"v\"
        run_timeout_hours = 30
    ";

    // Act
    let roster = TomlRoster.parse(text).unwrap();

    // Assert
    assert_eq!(roster.repos[0].run_timeout, Duration::from_hours(2));
    assert_eq!(roster.repos[1].run_timeout, Duration::from_hours(30));
    let zero = TomlRoster
        .parse("work_dir = \"/w\"\nresults_url = \"r\"\n[[repo]]\nname = \"a\"\nurl = \"u\"\nrun_timeout_hours = 0")
        .unwrap_err()
        .0;
    assert!(zero.contains("positive"), "{zero}");
}

#[test]
fn parse_run_unit_names_the_unit_runs_go_through() {
    // Arrange
    let none = "work_dir = \"/w\"\nresults_url = \"r\"\n[[repo]]\nname = \"a\"\nurl = \"u\"";
    let named = "work_dir = \"/w\"\nresults_url = \"r\"\nrun_unit = \"sentinel4rust-mutants\"\n[[repo]]\nname = \"a\"\nurl = \"u\"";
    let odd = "work_dir = \"/w\"\nresults_url = \"r\"\nrun_unit = \"x; rm -rf\"\n[[repo]]\nname = \"a\"\nurl = \"u\"";

    // Act
    let direct = TomlRoster.parse(none).unwrap();
    let isolated = TomlRoster.parse(named).unwrap();
    let refused = TomlRoster.parse(odd).unwrap_err().0;

    // Assert
    assert_eq!(direct.run_unit, None);
    assert_eq!(
        isolated.run_unit,
        Some(String::from("sentinel4rust-mutants"))
    );
    assert!(refused.contains("run_unit"), "{refused}");
}

#[test]
fn parse_stale_after_hours_defaults_to_a_day() {
    // Arrange
    let minimal = "work_dir = \"/w\"\nresults_url = \"r\"\n[[repo]]\nname = \"a\"\nurl = \"u\"";
    let set = "work_dir = \"/w\"\nresults_url = \"r\"\nstale_after_hours = 6\n[[repo]]\nname = \"a\"\nurl = \"u\"";

    // Act
    let default = TomlRoster.parse(minimal).unwrap();
    let chosen = TomlRoster.parse(set).unwrap();

    // Assert
    assert_eq!(default.stale_after, Duration::from_hours(24));
    assert_eq!(chosen.stale_after, Duration::from_hours(6));
}
