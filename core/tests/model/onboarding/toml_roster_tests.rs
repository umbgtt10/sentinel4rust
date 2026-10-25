// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::toml_roster::TomlRoster;
use std::path::PathBuf;
use std::time::Duration;

const FULL: &str = r#"
    work_dir = "/srv/s"
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
    assert_eq!(roster.work_dir, PathBuf::from("/srv/s"));
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
fn parse_invalid_rosters_fails_naming_the_problem() {
    // Arrange
    let repo = "[[repo]]\nname = \"a\"\nurl = \"u\"\n";
    let cases = [
        (repo.to_string(), "work_dir"),
        ("work_dir = \"/w\"".to_string(), "no [[repo]]"),
        (
            format!("work_dir = \"/w\"\ntypo = 1\n{repo}"),
            "unknown key",
        ),
        (format!("work_dir = \"/w\"\n{repo}{repo}"), "duplicate name"),
        (format!("work_dir = \"/w\"\njobs = 0\n{repo}"), "positive"),
        (
            format!("work_dir = \"/w\"\n{repo}branch = \"mutants\""),
            "invalid branch",
        ),
        (
            "work_dir = \"/w\"\n[[repo]]\nname = \"../x\"\nurl = \"u\"".to_string(),
            "invalid name",
        ),
        (
            "work_dir = \"/w\"\n[[repo]]\nname = \"a\"\nurl = \" \"".to_string(),
            "empty url",
        ),
        (
            format!("work_dir = \"/w\"\n{repo}[[other]]\nx = 1"),
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
        .parse("work_dir = \"/w\"\n[[repo]]\nname = \"a\"\nurl = \"u\"")
        .unwrap();

    // Assert
    assert_eq!(roster.pause, Duration::from_secs(300));
    assert_eq!(roster.jobs, 1);
    assert_eq!(roster.run_timeout, Duration::from_secs(12 * 3600));
    assert_eq!(roster.config_version, 1);
}

#[test]
fn parse_repo_run_timeout_overrides_the_global_cap() {
    // Arrange
    let text = "\
        work_dir = \"/w\"
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
    assert_eq!(roster.repos[0].run_timeout, Duration::from_secs(2 * 3600));
    assert_eq!(roster.repos[1].run_timeout, Duration::from_secs(30 * 3600));
    let zero = TomlRoster
        .parse("work_dir = \"/w\"\n[[repo]]\nname = \"a\"\nurl = \"u\"\nrun_timeout_hours = 0")
        .unwrap_err()
        .0;
    assert!(zero.contains("positive"), "{zero}");
}

#[test]
fn parse_stale_after_hours_defaults_to_a_day() {
    // Arrange
    let minimal = "work_dir = \"/w\"\n[[repo]]\nname = \"a\"\nurl = \"u\"";
    let set = "work_dir = \"/w\"\nstale_after_hours = 6\n[[repo]]\nname = \"a\"\nurl = \"u\"";

    // Act
    let default = TomlRoster.parse(minimal).unwrap();
    let chosen = TomlRoster.parse(set).unwrap();

    // Assert
    assert_eq!(default.stale_after, Duration::from_secs(24 * 3600));
    assert_eq!(chosen.stale_after, Duration::from_secs(6 * 3600));
}
