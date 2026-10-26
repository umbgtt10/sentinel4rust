// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One repository's visits and back-off.

use core::time::Duration;

use crate::model::commit_hash::CommitHash;
use crate::model::timestamp::Timestamp;
use crate::patrol::patrol_constants::MAX_BACKOFF;

/// What the last visit found wrong that waits out the cap and is logged once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trouble {
    Unreachable,
    Unreadable,
}

/// When one repository was last visited, how long it is backing off, its tool failures on one commit, and
/// whether it was last found unreachable or its record unreadable.
#[derive(Debug, Default, Clone)]
pub struct RepoState {
    last_visit: Option<Timestamp>,
    backoff_until: Option<Timestamp>,
    consecutive_failures: u32,
    tool_failures: Option<(CommitHash, u32)>,
    trouble: Option<Trouble>,
}

impl RepoState {
    /// Forgets tool failures (after a publish, or when the commit is current).
    pub fn clear_tool_failures(&mut self) {
        self.tool_failures = None;
    }

    /// When the repository is next due, `pause` after its last visit and not before its back-off ends; `None`
    /// means now (never visited, never failed).
    pub fn due_at(&self, pause: Duration) -> Option<Timestamp> {
        let after_pause = self.last_visit.map(|t| t.plus(pause));
        match (after_pause, self.backoff_until) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }

    /// Backs off after a failure: `pause × 2^(failures-1)`, capped at [`MAX_BACKOFF`].
    pub fn failed(&mut self, pause: Duration, now: Timestamp) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.trouble = None;
        let factor = 2u32.saturating_pow(self.consecutive_failures - 1);
        let wait = pause.saturating_mul(factor).min(MAX_BACKOFF);
        self.backoff_until = Some(now.plus(wait));
    }

    /// Whether the last visit found the repository unreachable.
    pub fn is_unreachable(&self) -> bool {
        self.trouble == Some(Trouble::Unreachable)
    }

    /// Whether the last visit found the record on `mutants` unreadable.
    pub fn is_unreadable(&self) -> bool {
        self.trouble == Some(Trouble::Unreadable)
    }

    /// Clears the back-off after anything but a failure.
    pub fn succeeded(&mut self) {
        self.consecutive_failures = 0;
        self.backoff_until = None;
        self.trouble = None;
    }

    /// Counts a tool failure on `head`; returns how many there have been on that commit.
    pub fn tool_failure(&mut self, head: &CommitHash) -> u32 {
        let count = match &self.tool_failures {
            Some((commit, n)) if commit == head => n + 1,
            _ => 1,
        };
        self.tool_failures = Some((head.clone(), count));
        count
    }

    /// Backs off straight to the cap; returns whether this is news (it was reachable before).
    pub fn unreachable(&mut self, now: Timestamp) -> bool {
        self.troubled(Trouble::Unreachable, now)
    }

    /// Backs off straight to the cap after an unreadable record; returns whether this is news.
    pub fn unreadable(&mut self, now: Timestamp) -> bool {
        self.troubled(Trouble::Unreadable, now)
    }

    /// Records the end of a visit.
    pub fn visited(&mut self, now: Timestamp) {
        self.last_visit = Some(now);
    }

    fn troubled(&mut self, trouble: Trouble, now: Timestamp) -> bool {
        let news = self.trouble != Some(trouble);
        self.trouble = Some(trouble);
        self.backoff_until = Some(now.plus(MAX_BACKOFF));
        news
    }
}
