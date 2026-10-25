// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::world::World;
use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::run_record::RunRecord;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::model::snapshot::Snapshot;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::status_line::StatusLine;
use sentinel4rust::patrol::status_report::StatusReport;
use std::rc::Rc;
use std::time::Duration;

const COMMITTED: i64 = 1_792_800_000;
const DAY: Duration = Duration::from_secs(24 * 3600);

fn hash(c: char) -> CommitHash {
    CommitHash::parse(&c.to_string().repeat(40)).unwrap()
}

fn line(text: &str, needs_attention: bool) -> StatusLine {
    StatusLine {
        text: text.to_string(),
        needs_attention,
    }
}

fn record_for(commit: &CommitHash) -> RecordState {
    RecordState::Present(Box::new(RunRecord {
        commit: commit.clone(),
        config_version: 1,
        kind: KIND_FULL.to_string(),
        outcome: OutcomeKind::Completed,
        started: Timestamp::from_unix(0),
        finished: Timestamp::from_unix(0),
        counts: MutantCounts::default(),
        new: 0,
        gone: 0,
        cargo_mutants_version: String::new(),
        rustc_version: String::new(),
        arguments: String::new(),
        reason: None,
        log_tail: None,
    }))
}

fn repo(name: &str) -> OnboardedRepo {
    OnboardedRepo {
        name: name.to_string(),
        url: "u".to_string(),
        branch: "main".to_string(),
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
    }
}

fn report(world: &Rc<World>) -> StatusReport {
    StatusReport::new(world.clone(), world.clone(), 1, DAY)
}

fn snapshot(head: &CommitHash, record: RecordState) -> Snapshot {
    Snapshot {
        head: head.clone(),
        record,
        previous_missed: vec![],
        readiness: Ok(()),
    }
}

// Waiting since `committed` for `waited` seconds.
fn world(waited: i64) -> Rc<World> {
    let world = Rc::new(World::default());
    world.committed.set(Some(COMMITTED));
    world.now.set(COMMITTED + waited);
    world
}

#[test]
fn check_of_a_current_record_says_current_and_needs_nothing() {
    // Arrange
    let world = world(90_000);
    world.queue_sync(Ok(snapshot(&hash('a'), record_for(&hash('a')))));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            "repo=a state=current main=aaaaaaa record=aaaaaaa",
            false
        )]
    );
}

#[test]
fn check_of_a_moved_main_past_the_limit_needs_attention() {
    // Arrange
    let world = world(90_000);
    world.queue_sync(Ok(snapshot(&hash('b'), record_for(&hash('a')))));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            "repo=a state=behind main=bbbbbbb record=aaaaaaa waited_s=90000",
            true
        )]
    );
}

#[test]
fn check_of_a_moved_main_within_the_limit_says_behind_and_how_long() {
    // Arrange
    let world = world(3600);
    world.queue_sync(Ok(snapshot(&hash('b'), record_for(&hash('a')))));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            "repo=a state=behind main=bbbbbbb record=aaaaaaa waited_s=3600",
            false
        )]
    );
}

#[test]
fn check_of_an_unknown_commit_time_needs_attention() {
    // Arrange
    let world = world(60);
    world.committed.set(None);
    world.queue_sync(Ok(snapshot(&hash('b'), RecordState::Absent)));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            "repo=a state=no-record main=bbbbbbb waited_s=unknown",
            true
        )]
    );
}

#[test]
fn check_of_an_unreachable_repository_is_unknown_and_needs_attention() {
    // Arrange
    let world = world(60);
    world.queue_sync(Err(SyncError::Unreachable(
        "git fetch: exited with 128: Repository not found".to_string(),
    )));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            r#"repo=a state=unknown reason="git fetch: exited with 128: Repository not found""#,
            true
        )]
    );
}

#[test]
fn check_of_an_unreadable_record_says_so_with_the_reason() {
    // Arrange
    let world = world(60);
    world.queue_sync(Ok(snapshot(
        &hash('b'),
        RecordState::Unreadable(RecordError::Invalid("line 1".to_string())),
    )));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            r#"repo=a state=unreadable main=bbbbbbb reason="invalid summary: line 1" waited_s=60"#,
            false
        )]
    );
}

#[test]
fn check_of_an_unready_repository_says_skipped_and_needs_nothing() {
    // Arrange
    let world = world(90_000);
    world.queue_sync(Ok(Snapshot {
        readiness: Err(SkipReason::NoFilters),
        ..snapshot(&hash('a'), RecordState::Absent)
    }));

    // Act
    let lines = report(&world).check(&[repo("a")]);

    // Assert
    assert_eq!(
        lines,
        [line(
            "repo=a state=skipped main=aaaaaaa reason=no-filters",
            false
        )]
    );
}

#[test]
fn check_of_several_repositories_reports_each_in_roster_order() {
    // Arrange
    let world = world(60);
    world.queue_sync(Ok(snapshot(&hash('a'), record_for(&hash('a')))));
    world.queue_sync(Ok(snapshot(&hash('b'), RecordState::Absent)));

    // Act
    let lines = report(&world).check(&[repo("a"), repo("b")]);

    // Assert
    assert_eq!(
        lines,
        [
            line("repo=a state=current main=aaaaaaa record=aaaaaaa", false),
            line("repo=b state=no-record main=bbbbbbb waited_s=60", false),
        ]
    );
}
