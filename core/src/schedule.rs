// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Which repository to visit next, and the in-memory back-off (design 1, 2.6).
//!
//! A repository is due when `pause` has passed since its last visit and it is not backing off. Among the due
//! repositories, the one that has waited longest goes first (ties in roster order), so a repository whose
//! `main` keeps moving cannot starve the rest. Losing this state on a restart costs at most an early visit.

use std::collections::HashMap;
use std::time::Duration;

use crate::model::{CommitHash, OnboardedRepo, Timestamp};

/// Cap on the back-off, and the wait after "unreachable".
pub const MAX_BACKOFF: Duration = Duration::from_secs(6 * 3600);
/// Tool failures on one commit before `tool-failed` is published.
pub const MAX_TOOL_FAILURES: u32 = 3;

#[derive(Debug, Default, Clone)]
struct RepoState {
    last_visit: Option<Timestamp>,
    backoff_until: Option<Timestamp>,
    consecutive_failures: u32,
    tool_failures: Option<(CommitHash, u32)>,
    unreachable: bool,
}

/// What to do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// Visit the repository at this roster index.
    Visit(usize),
    /// Nothing is due before this instant.
    SleepUntil(Timestamp),
}

/// Per-repository visit times and back-off.
#[derive(Debug)]
pub struct Schedule {
    pause: Duration,
    states: HashMap<String, RepoState>,
}

impl Schedule {
    /// A schedule visiting each repository at most once per `pause`.
    pub fn new(pause: Duration) -> Self {
        Self {
            pause,
            states: HashMap::new(),
        }
    }

    fn state(&mut self, name: &str) -> &mut RepoState {
        self.states.entry(name.to_string()).or_default()
    }

    /// When the repository is next due; `None` means now (never visited).
    fn due_at(&self, name: &str) -> Option<Timestamp> {
        let state = self.states.get(name)?;
        let after_pause = state.last_visit.map(|t| t.plus(self.pause));
        match (after_pause, state.backoff_until) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }

    /// The next repository to visit, or how long to sleep.
    pub fn next(&self, repos: &[OnboardedRepo], now: Timestamp) -> Next {
        let due: Vec<(usize, Option<Timestamp>)> = repos
            .iter()
            .enumerate()
            .map(|(i, repo)| (i, self.due_at(&repo.name)))
            .collect();
        let ready = due
            .iter()
            .filter(|(_, at)| at.is_none_or(|t| t <= now))
            .min_by_key(|(i, at)| (*at, *i));
        match ready {
            Some((i, _)) => Next::Visit(*i),
            None => Next::SleepUntil(
                due.iter()
                    .filter_map(|(_, at)| *at)
                    .min()
                    .unwrap_or_else(|| now.plus(self.pause)),
            ),
        }
    }

    /// Records the end of a visit.
    pub fn visited(&mut self, name: &str, now: Timestamp) {
        self.state(name).last_visit = Some(now);
    }

    /// Clears the back-off after anything but a failure.
    pub fn succeeded(&mut self, name: &str) {
        let state = self.state(name);
        state.consecutive_failures = 0;
        state.backoff_until = None;
        state.unreachable = false;
    }

    /// Backs off after a failure: `pause × 2^(failures-1)`, capped at [`MAX_BACKOFF`].
    pub fn failed(&mut self, name: &str, now: Timestamp) {
        let pause = self.pause;
        let state = self.state(name);
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        state.unreachable = false;
        let factor = 2u32.saturating_pow(state.consecutive_failures - 1);
        let wait = pause.saturating_mul(factor).min(MAX_BACKOFF);
        state.backoff_until = Some(now.plus(wait));
    }

    /// Backs off straight to the cap; returns whether this is news (it was reachable before).
    pub fn unreachable(&mut self, name: &str, now: Timestamp) -> bool {
        let state = self.state(name);
        let news = !state.unreachable;
        state.unreachable = true;
        state.backoff_until = Some(now.plus(MAX_BACKOFF));
        news
    }

    /// Whether the last visit found the repository unreachable.
    pub fn is_unreachable(&self, name: &str) -> bool {
        self.states.get(name).is_some_and(|s| s.unreachable)
    }

    /// Counts a tool failure on `head`; returns how many there have been on that commit.
    pub fn tool_failure(&mut self, name: &str, head: &CommitHash) -> u32 {
        let state = self.state(name);
        let count = match &state.tool_failures {
            Some((commit, n)) if commit == head => n + 1,
            _ => 1,
        };
        state.tool_failures = Some((head.clone(), count));
        count
    }

    /// Forgets tool failures (after a publish, or when the commit is current).
    pub fn clear_tool_failures(&mut self, name: &str) {
        self.state(name).tool_failures = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn everything_is_due_at_start_in_roster_order() {
        let s = Schedule::new(PAUSE);
        assert_eq!(s.next(&repos(&["a", "b"]), t(0)), Next::Visit(0));
    }

    #[test]
    fn no_repository_is_visited_twice_within_the_pause() {
        let r = repos(&["a", "b"]);
        let mut s = Schedule::new(PAUSE);
        s.visited("a", t(0));
        assert_eq!(s.next(&r, t(1)), Next::Visit(1));
        s.visited("b", t(10));
        assert_eq!(s.next(&r, t(20)), Next::SleepUntil(t(300)));
        assert_eq!(s.next(&r, t(300)), Next::Visit(0));
    }

    #[test]
    fn the_longest_waiting_due_repository_goes_first() {
        let r = repos(&["a", "b", "c"]);
        let mut s = Schedule::new(PAUSE);
        s.visited("c", t(0));
        s.visited("b", t(100));
        s.visited("a", t(200));
        assert_eq!(s.next(&r, t(10_000)), Next::Visit(2));
    }

    #[test]
    fn back_off_doubles_up_to_the_cap_and_resets() {
        let r = repos(&["a"]);
        let mut s = Schedule::new(PAUSE);
        let mut expected = PAUSE;
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
    fn unreachable_waits_the_cap_and_is_news_once() {
        let r = repos(&["a"]);
        let mut s = Schedule::new(PAUSE);
        assert!(s.unreachable("a", t(0)));
        assert!(!s.unreachable("a", t(0)));
        assert!(s.is_unreachable("a"));
        assert_eq!(s.next(&r, t(0)), Next::SleepUntil(t(0).plus(MAX_BACKOFF)));
        s.succeeded("a");
        assert!(!s.is_unreachable("a"));
    }

    #[test]
    fn tool_failures_count_per_commit() {
        let h1 = CommitHash::parse(&"1".repeat(40)).unwrap();
        let h2 = CommitHash::parse(&"2".repeat(40)).unwrap();
        let mut s = Schedule::new(PAUSE);
        assert_eq!(s.tool_failure("a", &h1), 1);
        assert_eq!(s.tool_failure("a", &h1), 2);
        assert_eq!(s.tool_failure("a", &h2), 1);
        s.clear_tool_failures("a");
        assert_eq!(s.tool_failure("a", &h2), 1);
    }
}
