// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Discovery, run again when it is due (`ADR-GithubAppDiscovery`).

use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::roster::Roster;
use crate::model::timestamp::Timestamp;
use crate::patrol::journal_line::JournalLine;
use crate::patrol::watch_list::WatchList;
use crate::traits::journal::Journal;
use crate::traits::repo_discovery::RepoDiscovery;

/// The repositories watched -- the roster's, and what discovery last found -- and when discovery runs next.
pub struct DiscoveryRefresh {
    discovery: Option<Rc<dyn RepoDiscovery>>,
    journal: Rc<dyn Journal>,
    watch: WatchList,
    interval: Duration,
    retry: Duration,
    due: Option<Timestamp>,
    repos: Vec<OnboardedRepo>,
}

impl DiscoveryRefresh {
    /// For `roster`: due at once when there is a `discovery`, never when there is none. A failure is tried again
    /// after the roster's pause.
    pub fn new(
        roster: &Roster,
        discovery: Option<Rc<dyn RepoDiscovery>>,
        journal: Rc<dyn Journal>,
    ) -> Self {
        Self {
            due: discovery.as_ref().map(|_| Timestamp::from_unix(0)),
            discovery,
            journal,
            watch: WatchList::new(roster),
            interval: roster.discovery_interval,
            retry: roster.pause,
            repos: roster.repos.clone(),
        }
    }

    /// When discovery runs next; `None`: never.
    pub fn due(&self) -> Option<Timestamp> {
        self.due
    }

    /// Runs discovery when it is due; whether the list was made again. One journal line either way.
    pub fn refresh(&mut self, now: Timestamp) -> bool {
        let (Some(discovery), Some(due)) = (self.discovery.clone(), self.due) else {
            return false;
        };
        if now < due {
            return false;
        }
        match discovery.discover() {
            Ok(found) => {
                let repos = self.watch.merge(&found);
                let names: Vec<String> = repos.iter().map(|repo| repo.name.clone()).collect();
                let before: Vec<String> = self.repos.iter().map(|repo| repo.name.clone()).collect();
                let added: Vec<String> = names
                    .iter()
                    .filter(|name| !before.contains(name))
                    .cloned()
                    .collect();
                let removed: Vec<String> = before
                    .iter()
                    .filter(|name| !names.contains(name))
                    .cloned()
                    .collect();
                self.journal
                    .line(&JournalLine.discovery(repos.len(), &added, &removed));
                self.repos = repos;
                self.due = Some(now.plus(self.interval));
                true
            }
            Err(error) => {
                self.journal.line(&JournalLine.discovery_failed(&error));
                self.due = Some(now.plus(self.retry));
                false
            }
        }
    }

    /// The repositories watched now, the roster's first.
    pub fn repos(&self) -> &[OnboardedRepo] {
        &self.repos
    }
}
