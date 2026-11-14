// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Global settings and the onboarded repositories.

use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::model::onboarding::github_app::GithubApp;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;

/// Global settings and the onboarded repositories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roster {
    /// Clones, worktrees and cargo-mutants output live here.
    pub work_dir: String,
    /// Polling interval per repository.
    pub pause: Duration,
    /// Default cargo-mutants `--jobs`.
    pub jobs: u32,
    /// Cap on one cargo-mutants run, for every repository without its own.
    pub run_timeout: Duration,
    /// Bump to rerun every repository.
    pub config_version: i64,
    /// How long `main`'s head may wait for a record before `--status` calls the repository stale.
    pub stale_after: Duration,
    /// The shared repository every publication is pushed to, one folder per watched repository
    /// (`ADR-SharedResultsRepository`).
    pub results_url: String,
    /// The systemd template unit cargo-mutants runs in, away from the deploy keys; `None`: in the service itself.
    pub run_unit: Option<String>,
    /// The GitHub App repositories are discovered through; `None`: only the `[[repo]]` entries are watched.
    pub github_app: Option<GithubApp>,
    /// How often discovery runs again.
    pub discovery_interval: Duration,
    /// The repositories named in the roster, in roster order; with a GitHub App, they come before and win over
    /// discovered ones.
    pub repos: Vec<OnboardedRepo>,
}
