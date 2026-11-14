// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `RepoSource` and `ResultStore` by shelling out to `git` (`ADR-SharedResultsRepository`,
//! `ADR-ExactCommitOnlyMutantsPushed`). Credentials stay with the system's git and SSH configuration.
//!
//! Each watched repository is a bare clone at `<work_dir>/repos/<name>.git` whose branch is fetched into
//! `refs/remotes/origin/*` with an explicit refspec; nothing is ever pushed there. Runs use a detached worktree at
//! `<work_dir>/runs/<name>`. The results repository is one more bare clone, `<work_dir>/results.git`: records are
//! read from its fetched `main`, one folder per watched repository, and publishing never checks anything out --
//! blobs, a tree in a temporary index and a commit are written with plumbing commands, and only its `main` is
//! pushed.

pub mod bare_clone;
pub mod crate_readiness;
pub mod detached_worktree;
pub mod git_cli;
pub mod git_constants;
pub mod git_failure;
pub mod git_runner;
pub mod plumbing_commit;
pub mod record_reader;
pub mod results_clone;
