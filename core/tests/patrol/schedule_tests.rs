// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::next::Next;
use sentinel4rust::patrol::patrol_constants::MAX_BACKOFF;
use sentinel4rust::patrol::schedule::Schedule;
use std::time::Duration;

const PAUSE: Duration = Duration::from_secs(300);

fn repos(names: &[&str]) -> Vec<OnboardedRepo> {
    names
        .iter()
        .map(|n| OnboardedRepo {
            name: (*n).to_string(),
            url: String::new(),
            branch: "main".to_string(),
            jobs: 1,
        })
        .collect()
}

fn t(seconds: i64) -> Timestamp {
    Timestamp::from_unix(seconds)
}

#[test]
fn failed_repeatedly_doubles_the_back_off_to_the_cap_and_resets() {
    // Arrange
    let r = repos(&["a"]);
    let mut s = Schedule::new(PAUSE);
    let mut expected = PAUSE;

    // Act & Assert
    for _ in 0..12 {
        s.failed("a", t(0));
        s.visited("a", t(0));
        let wait = expected.max(PAUSE).min(MAX_BACKOFF);
        assert_eq!(s.next(&r, t(0)), Next::SleepUntil(t(0).plus(wait)));
        expected *= 2;
    }
    s.succeeded("a");
    assert_eq!(s.next(&r, t(0)), Next::SleepUntil(t(300)));
}

#[test]
fn next_at_start_visits_in_roster_order() {
    // Arrange
    let s = Schedule::new(PAUSE);

    // Act
    let next = s.next(&repos(&["a", "b"]), t(0));

    // Assert
    assert_eq!(next, Next::Visit(0));
}

#[test]
fn next_with_several_due_picks_the_longest_waiting() {
    // Arrange
    let r = repos(&["a", "b", "c"]);
    let mut s = Schedule::new(PAUSE);
    s.visited("c", t(0));
    s.visited("b", t(100));
    s.visited("a", t(200));

    // Act
    let next = s.next(&r, t(10_000));

    // Assert
    assert_eq!(next, Next::Visit(2));
}

#[test]
fn next_within_the_pause_skips_a_visited_repository() {
    // Arrange
    let r = repos(&["a", "b"]);
    let mut s = Schedule::new(PAUSE);

    // Act & Assert
    s.visited("a", t(0));
    assert_eq!(s.next(&r, t(1)), Next::Visit(1));
    s.visited("b", t(10));
    assert_eq!(s.next(&r, t(20)), Next::SleepUntil(t(300)));
    assert_eq!(s.next(&r, t(300)), Next::Visit(0));
}

#[test]
fn tool_failure_counts_per_commit() {
    // Arrange
    let h1 = CommitHash::parse(&"1".repeat(40)).unwrap();
    let h2 = CommitHash::parse(&"2".repeat(40)).unwrap();
    let mut s = Schedule::new(PAUSE);

    // Act & Assert
    assert_eq!(s.tool_failure("a", &h1), 1);
    assert_eq!(s.tool_failure("a", &h1), 2);
    assert_eq!(s.tool_failure("a", &h2), 1);
    s.clear_tool_failures("a");
    assert_eq!(s.tool_failure("a", &h2), 1);
}

#[test]
fn unreachable_waits_the_cap_and_is_news_once() {
    // Arrange
    let r = repos(&["a"]);
    let mut s = Schedule::new(PAUSE);

    // Act & Assert
    assert!(s.unreachable("a", t(0)));
    assert!(!s.unreachable("a", t(0)));
    assert!(s.is_unreachable("a"));
    assert_eq!(s.next(&r, t(0)), Next::SleepUntil(t(0).plus(MAX_BACKOFF)));
    s.succeeded("a");
    assert!(!s.is_unreachable("a"));
}

#[test]
fn unreadable_waits_the_cap_and_is_news_once() {
    // Arrange
    let r = repos(&["a"]);
    let mut s = Schedule::new(PAUSE);

    // Act & Assert
    assert!(s.unreadable("a", t(0)));
    assert!(!s.unreadable("a", t(0)));
    assert!(s.is_unreadable("a"));
    assert_eq!(s.next(&r, t(0)), Next::SleepUntil(t(0).plus(MAX_BACKOFF)));
    s.succeeded("a");
    assert!(!s.is_unreadable("a"));
}
