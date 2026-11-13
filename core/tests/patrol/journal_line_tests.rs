// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::onboarding::roster::Roster;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::round_step::RoundStep;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::journal_line::JournalLine;
use std::time::Duration;

fn published(caught: u64, timeout: u64) -> RoundStep {
    RoundStep::Published {
        head: CommitHash::parse(&"ab".repeat(20)).unwrap(),
        outcome: OutcomeKind::Completed,
        counts: MutantCounts {
            caught,
            missed: 2,
            timeout,
            unviable: 3,
        },
        new: 1,
        gone: 0,
    }
}

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
        work_dir: String::from("/w"),
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
fn visit_published_names_its_counts() {
    // Arrange & Act
    let line = JournalLine.visit(&repo("a"), &published(10, 1), Duration::from_secs(7));

    // Assert
    assert_eq!(
        line,
        "repo=a step=published hash=abababa outcome=completed missed=2 caught=10 timeout=1 unviable=3 new=1 gone=0 duration_s=7"
    );
}

// A run whose mutants mostly timed out says so on its own line, before anyone
// reads its lists.
#[test]
fn visit_published_with_more_timeouts_than_caught_warns() {
    // Arrange & Act
    let line = JournalLine.visit(&repo("a"), &published(74, 121), Duration::from_secs(7));

    // Assert
    assert_eq!(
        line,
        "repo=a step=published hash=abababa outcome=completed missed=2 caught=74 timeout=121 unviable=3 new=1 gone=0 warning=timeouts-outnumber-caught duration_s=7"
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
