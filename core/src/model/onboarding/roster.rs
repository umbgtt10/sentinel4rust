// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Global settings and the onboarded repositories.

use std::path::PathBuf;
use std::time::Duration;

use crate::model::onboarding::onboarded_repo::OnboardedRepo;

/// Global settings and the onboarded repositories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roster {
    /// Clones, worktrees and cargo-mutants output live here.
    pub work_dir: PathBuf,
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
    /// The repositories, in roster order.
    pub repos: Vec<OnboardedRepo>,
}
