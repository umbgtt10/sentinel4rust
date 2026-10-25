// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Which repository to visit next.

use std::collections::HashMap;
use std::time::Duration;

use crate::model::commit_hash::CommitHash;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::timestamp::Timestamp;
use crate::patrol::next::Next;
use crate::patrol::repo_state::RepoState;

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

    /// Forgets the repository's tool failures (after a publish, or when the commit is current).
    pub fn clear_tool_failures(&mut self, name: &str) {
        self.update(name, RepoState::clear_tool_failures);
    }

    /// Backs the repository off after a failure.
    pub fn failed(&mut self, name: &str, now: Timestamp) {
        let pause = self.pause;
        self.update(name, |state| state.failed(pause, now));
    }

    /// Whether the last visit found the repository unreachable.
    pub fn is_unreachable(&self, name: &str) -> bool {
        self.states.get(name).is_some_and(RepoState::is_unreachable)
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

    /// Clears the repository's back-off after anything but a failure.
    pub fn succeeded(&mut self, name: &str) {
        self.update(name, RepoState::succeeded);
    }

    /// Counts a tool failure on `head`; returns how many there have been on that commit.
    pub fn tool_failure(&mut self, name: &str, head: &CommitHash) -> u32 {
        self.update(name, |state| state.tool_failure(head))
    }

    /// Backs the repository off straight to the cap; returns whether this is news.
    pub fn unreachable(&mut self, name: &str, now: Timestamp) -> bool {
        self.update(name, |state| state.unreachable(now))
    }

    /// Records the end of a visit.
    pub fn visited(&mut self, name: &str, now: Timestamp) {
        self.update(name, |state| state.visited(now));
    }

    fn due_at(&self, name: &str) -> Option<Timestamp> {
        self.states.get(name)?.due_at(self.pause)
    }

    fn update<T>(&mut self, name: &str, change: impl FnOnce(&mut RepoState) -> T) -> T {
        change(self.states.entry(name.to_string()).or_default())
    }
}
