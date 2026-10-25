// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `RepoSource` and `ResultStore` by shelling out to `git` (design 2.1, 2.4). Credentials stay with the
//! system's git and SSH configuration.
//!
//! Each repository is a bare clone at `<work_dir>/repos/<name>.git` whose remote branches are fetched into
//! `refs/remotes/origin/*` with explicit refspecs. Runs use a detached worktree at `<work_dir>/runs/<name>`.
//! Publishing never checks anything out: blobs, a tree in a temporary index, and a commit are written with
//! plumbing commands, and only `refs/heads/mutants` is pushed.

pub mod bare_clone;
pub mod crate_readiness;
pub mod detached_worktree;
pub mod git_cli;
pub mod git_constants;
pub mod git_failure;
pub mod git_runner;
pub mod plumbing_commit;
pub mod record_reader;
pub mod remote_listing;
pub mod unreachable_classifier;
