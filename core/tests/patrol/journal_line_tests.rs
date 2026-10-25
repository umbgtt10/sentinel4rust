// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::onboarding::roster::Roster;
use sentinel4rust::model::round_step::RoundStep;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::journal_line::JournalLine;
use std::path::PathBuf;
use std::time::Duration;

fn repo(name: &str) -> OnboardedRepo {
    OnboardedRepo {
        name: name.to_string(),
        url: format!("git@x:{name}.git"),
        branch: "main".to_string(),
        jobs: 4,
        run_timeout: Duration::from_secs(3600),
    }
}

#[test]
fn startup_names_the_version_repositories_pause_and_config_version() {
    // Arrange
    let roster = Roster {
        work_dir: PathBuf::from("/w"),
        pause: Duration::from_secs(300),
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
        config_version: 2,
        stale_after: Duration::from_secs(24 * 3600),
        run_unit: None,
        repos: vec![repo("a"), repo("b")],
    };

    // Act
    let line = JournalLine.startup(&roster, Timestamp::from_unix(1_792_800_000));

    // Assert
    assert_eq!(
        line,
        format!(
            "sentinel4rust {} started at {}: 2 repositories, pause 300s, config_version 2",
            env!("CARGO_PKG_VERSION"),
            Timestamp::from_unix(1_792_800_000)
        )
    );
}

#[test]
fn visit_skipped_for_a_missing_toolchain_quotes_rustups_message() {
    // Arrange
    let step = RoundStep::Skipped {
        reason: SkipReason::ToolchainMissing {
            message:
                "exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed"
                    .to_string(),
        },
    };

    // Act
    let line = JournalLine.visit(&repo("a"), &step, Duration::from_secs(1));

    // Assert
    assert_eq!(
        line,
        r#"repo=a step=skipped reason=toolchain-missing message="exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed" duration_s=1"#
    );
}

#[test]
fn visit_skipped_for_a_path_outside_names_the_manifest() {
    // Arrange
    let step = RoundStep::Skipped {
        reason: SkipReason::PathOutsideRepo {
            manifest: "crates/a/Cargo.toml".to_string(),
        },
    };

    // Act
    let line = JournalLine.visit(&repo("a"), &step, Duration::from_secs(1));

    // Assert
    assert_eq!(
        line,
        "repo=a step=skipped reason=path-outside-repo manifest=crates/a/Cargo.toml duration_s=1"
    );
}

#[test]
fn visit_unreadable_quotes_the_record_error() {
    // Arrange
    let step = RoundStep::Unreadable {
        reason: "invalid summary: line 3: expected `=`".to_string(),
    };

    // Act
    let line = JournalLine.visit(&repo("a"), &step, Duration::from_secs(1));

    // Assert
    assert_eq!(
        line,
        r#"repo=a step=unreadable reason="invalid summary: line 3: expected `=`" duration_s=1"#
    );
}

#[test]
fn visit_with_a_reason_quotes_it() {
    // Arrange
    let step = RoundStep::BackedOff {
        reason: "git fetch: exited with 128: fatal \"x\"".to_string(),
    };

    // Act
    let line = JournalLine.visit(&repo("a"), &step, Duration::from_secs(3));

    // Assert
    assert_eq!(
        line,
        r#"repo=a step=backed-off reason="git fetch: exited with 128: fatal \"x\"" duration_s=3"#
    );
}
