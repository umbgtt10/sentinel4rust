// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Which repositories are watched: the roster's own, then the discovered ones (`ADR-GithubAppDiscovery`).

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::model::github::discovered_repo::DiscoveredRepo;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::repo_names::RepoNames;
use crate::model::onboarding::roster::Roster;

/// The roster's repositories and defaults, ready to take in what discovery found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchList {
    explicit: Vec<OnboardedRepo>,
    jobs: u32,
    run_timeout: Duration,
    results_path: String,
}

impl WatchList {
    /// For `roster`.
    pub fn new(roster: &Roster) -> Self {
        Self {
            explicit: roster.repos.clone(),
            jobs: roster.jobs,
            run_timeout: roster.run_timeout,
            results_path: Self::owner_and_name(&roster.results_url),
        }
    }

    /// The roster's repositories in roster order, then every discovered one they do not name, by name, with the
    /// roster's defaults; never the results repository, nor a name that cannot name a clone.
    pub fn merge(&self, discovered: &[DiscoveredRepo]) -> Vec<OnboardedRepo> {
        let named: BTreeSet<&str> = self
            .explicit
            .iter()
            .map(|repo| repo.name.as_str())
            .collect();
        let mut found: Vec<&DiscoveredRepo> = discovered
            .iter()
            .filter(|repo| !named.contains(repo.name.as_str()))
            .filter(|repo| RepoNames.is_name(&repo.name))
            .filter(|repo| Self::owner_and_name(&repo.url) != self.results_path)
            .collect();
        found.sort_by(|a, b| a.name.cmp(&b.name));
        let mut repos = self.explicit.clone();
        repos.extend(found.into_iter().map(|repo| OnboardedRepo {
            name: repo.name.clone(),
            url: repo.url.clone(),
            branch: repo.branch.clone(),
            jobs: self.jobs,
            run_timeout: self.run_timeout,
        }));
        repos
    }

    // `owner/name` of a git URL, whether `https://host/owner/name.git` or `git@alias:owner/name.git`.
    fn owner_and_name(url: &str) -> String {
        let trimmed = url.trim_end_matches('/').trim_end_matches(".git");
        let parts: Vec<&str> = trimmed.rsplitn(3, ['/', ':']).collect();
        match parts.as_slice() {
            [name, owner, ..] => format!("{owner}/{name}"),
            _ => String::from(trimmed),
        }
    }
}
