// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::patrol_constants::MAX_BACKOFF;
use sentinel4rust::patrol::repo_state::RepoState;
use std::time::Duration;

const PAUSE: Duration = Duration::from_secs(300);

fn hash(c: char) -> CommitHash {
    CommitHash::parse(&c.to_string().repeat(40)).unwrap()
}

fn t(seconds: i64) -> Timestamp {
    Timestamp::from_unix(seconds)
}

#[test]
fn clear_tool_failures_restarts_the_count() {
    // Arrange
    let mut state = RepoState::default();
    state.tool_failure(&hash('a'));
    state.tool_failure(&hash('a'));

    // Act
    state.clear_tool_failures();

    // Assert
    assert_eq!(state.tool_failure(&hash('a')), 1);
}

#[test]
fn due_at_backing_off_past_the_pause_is_the_back_off() {
    // Arrange
    let mut state = RepoState::default();
    state.visited(t(0));
    state.failed(PAUSE, t(0));
    state.failed(PAUSE, t(0));

    // Act
    let due = state.due_at(PAUSE);

    // Assert
    assert_eq!(due, Some(t(600)));
}

#[test]
fn due_at_of_a_new_state_is_none() {
    // Arrange
    let state = RepoState::default();

    // Act
    let due = state.due_at(PAUSE);

    // Assert
    assert_eq!(due, None);
}

#[test]
fn failed_backs_off_a_pause_then_doubles_up_to_the_cap() {
    // Arrange
    let mut state = RepoState::default();

    // Act & Assert
    state.failed(PAUSE, t(0));
    assert_eq!(state.due_at(PAUSE), Some(t(300)));
    state.failed(PAUSE, t(0));
    assert_eq!(state.due_at(PAUSE), Some(t(600)));
    for _ in 0..10 {
        state.failed(PAUSE, t(0));
    }
    assert_eq!(state.due_at(PAUSE), Some(t(0).plus(MAX_BACKOFF)));
}

#[test]
fn is_unreachable_of_a_new_state_is_false() {
    // Arrange
    let state = RepoState::default();

    // Act
    let unreachable = state.is_unreachable();

    // Assert
    assert!(!unreachable);
}

#[test]
fn is_unreadable_of_a_new_state_is_false() {
    // Arrange
    let state = RepoState::default();

    // Act
    let unreadable = state.is_unreadable();

    // Assert
    assert!(!unreadable);
}

#[test]
fn succeeded_clears_the_back_off_and_unreachable() {
    // Arrange
    let mut state = RepoState::default();
    state.visited(t(0));
    state.unreachable(t(0));

    // Act
    state.succeeded();

    // Assert
    assert!(!state.is_unreachable());
    assert_eq!(state.due_at(PAUSE), Some(t(300)));
}

#[test]
fn succeeded_clears_unreadable() {
    // Arrange
    let mut state = RepoState::default();
    state.unreadable(t(0));

    // Act
    state.succeeded();

    // Assert
    assert!(!state.is_unreadable());
    assert_eq!(state.due_at(PAUSE), None);
}

#[test]
fn tool_failure_counts_on_one_commit_and_restarts_on_another() {
    // Arrange
    let mut state = RepoState::default();

    // Act
    let counts = [
        state.tool_failure(&hash('a')),
        state.tool_failure(&hash('a')),
        state.tool_failure(&hash('b')),
    ];

    // Assert
    assert_eq!(counts, [1, 2, 1]);
}

#[test]
fn unreachable_backs_off_six_hours() {
    // Arrange
    let mut state = RepoState::default();

    // Act
    state.unreachable(t(0));

    // Assert
    assert_eq!(state.due_at(PAUSE), Some(t(6 * 3600)));
}

#[test]
fn unreachable_backs_off_to_the_cap_and_is_news_once() {
    // Arrange
    let mut state = RepoState::default();

    // Act
    let news = [state.unreachable(t(0)), state.unreachable(t(10))];

    // Assert
    assert_eq!(news, [true, false]);
    assert!(state.is_unreachable());
    assert_eq!(state.due_at(PAUSE), Some(t(10).plus(MAX_BACKOFF)));
}

#[test]
fn unreadable_after_unreachable_is_news_and_ends_unreachable() {
    // Arrange
    let mut state = RepoState::default();
    state.unreachable(t(0));

    // Act
    let news = state.unreadable(t(10));

    // Assert
    assert!(news);
    assert!(state.is_unreadable());
    assert!(!state.is_unreachable());
}

#[test]
fn unreadable_backs_off_to_the_cap_and_is_news_once() {
    // Arrange
    let mut state = RepoState::default();

    // Act
    let news = [state.unreadable(t(0)), state.unreadable(t(10))];

    // Assert
    assert_eq!(news, [true, false]);
    assert!(state.is_unreadable());
    assert_eq!(state.due_at(PAUSE), Some(t(10).plus(MAX_BACKOFF)));
}

#[test]
fn visited_makes_it_due_a_pause_later() {
    // Arrange
    let mut state = RepoState::default();

    // Act
    state.visited(t(100));

    // Assert
    assert_eq!(state.due_at(PAUSE), Some(t(400)));
}
