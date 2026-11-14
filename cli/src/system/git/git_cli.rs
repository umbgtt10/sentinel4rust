// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `RepoSource` and `ResultStore` over git.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use crate::system::git::bare_clone::BareClone;
use crate::system::git::crate_readiness::CrateReadiness;
use crate::system::git::detached_worktree::DetachedWorktree;
use crate::system::git::git_constants::NETWORK_TIMEOUT;
use crate::system::git::git_runner::GitRunner;
use crate::system::git::results_clone::ResultsClone;
use crate::system::work_dir::WorkDir;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::checkout_error::CheckoutError;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::publish_error::PublishError;
use sentinel4rust::model::snapshot::Snapshot;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::protocol::git::remote_listing::RemoteListing;
use sentinel4rust::protocol::git::unreachable_classifier::UnreachableClassifier;
use sentinel4rust::traits::command_runner::CommandRunner;
use sentinel4rust::traits::repo_source::RepoSource;
use sentinel4rust::traits::result_store::ResultStore;

/// git, run through a [`CommandRunner`]: fetches and checks out the watched repositories, which it never pushes
/// to, and reads and publishes the records in the results repository.
pub struct GitCli {
    git: GitRunner,
    work_dir: WorkDir,
    clone: BareClone,
    readiness: CrateReadiness,
    worktree: DetachedWorktree,
    results: ResultsClone,
}

impl GitCli {
    /// A git client keeping its clones, checkouts and staging areas under `work_dir`, publishing to the results
    /// repository at `results_url`.
    pub fn new(runner: Rc<dyn CommandRunner>, work_dir: WorkDir, results_url: String) -> Self {
        let git = GitRunner::new(runner);
        Self {
            clone: BareClone::new(git.clone(), work_dir.clone()),
            readiness: CrateReadiness::new(git.clone()),
            worktree: DetachedWorktree::new(git.clone(), work_dir.clone()),
            results: ResultsClone::new(git.clone(), work_dir.clone(), results_url),
            git,
            work_dir,
        }
    }

    /// Removes leftovers of a crash: run checkouts, staging areas, stale worktree entries.
    pub fn prune(&self, repos: &[OnboardedRepo]) {
        for leftovers in [self.work_dir.runs(), self.work_dir.stages()] {
            let _ = fs::remove_dir_all(leftovers);
        }
        for repo in repos {
            self.clone.prune_worktrees(repo);
        }
    }

    // The branch is also kept at `refs/heads/<branch>`, where another repository's build can fetch it from the clone
    // through `file://`: a private git dependency, read in the run unit with no key (`docs/OPEN_POINTS.md` point 10).
    fn fetch(&self, dir: &Path, repo: &OnboardedRepo) -> Result<(), SyncError> {
        let branch_spec = format!(
            "+refs/heads/{}:refs/remotes/origin/{}",
            repo.branch, repo.branch
        );
        let served_spec = format!("+refs/heads/{}:refs/heads/{}", repo.branch, repo.branch);
        self.git
            .git(
                dir,
                &[
                    "fetch",
                    "--quiet",
                    "--no-tags",
                    "origin",
                    &branch_spec,
                    &served_spec,
                ],
                NETWORK_TIMEOUT,
            )
            .map(|_| ())
            .map_err(|r| UnreachableClassifier.classify(&r, "git fetch"))
    }

    fn head(&self, dir: &Path, repo: &OnboardedRepo) -> Result<CommitHash, SyncError> {
        let tip = format!("refs/remotes/origin/{}^{{commit}}", repo.branch);
        let text = self
            .git
            .local(dir, &["rev-parse", "--verify", &tip])
            .map_err(|failure| SyncError::Transient(failure.to_string()))?;
        CommitHash::parse(&text)
            .ok_or_else(|| SyncError::Transient(format!("bad commit hash `{}`", text.trim())))
    }
}

impl RepoSource for GitCli {
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError> {
        let dir = self.clone.ensure(repo)?;
        let branch_ref = format!("refs/heads/{}", repo.branch);
        let listing = self
            .git
            .git(
                &dir,
                &["ls-remote", "--heads", "origin", &branch_ref],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| UnreachableClassifier.classify(&r, "git ls-remote"))?;
        if !RemoteListing::parse(&listing).has(&branch_ref) {
            return Err(SyncError::NoBranch);
        }
        self.fetch(&dir, repo)?;
        let head = self.head(&dir, repo)?;
        self.results.fetch()?;
        let readiness = self.readiness.check(&dir, &head);
        Ok(Snapshot {
            head,
            record: self.results.record(repo),
            previous_missed: self.results.previous_missed(repo),
            readiness,
        })
    }

    fn checkout(&self, repo: &OnboardedRepo, head: &CommitHash) -> Result<String, CheckoutError> {
        self.remove_checkout(repo);
        self.worktree
            .add(repo, head)
            .map(|dir| dir.to_string_lossy().into_owned())
    }

    fn remove_checkout(&self, repo: &OnboardedRepo) {
        self.worktree.remove(repo);
        self.clone.prune_worktrees(repo);
    }

    fn committed_at(&self, repo: &OnboardedRepo, commit: &CommitHash) -> Option<Timestamp> {
        let dir = self.work_dir.bare_clone(repo);
        let seconds = self
            .git
            .local(&dir, &["show", "-s", "--format=%ct", commit.as_str()])
            .ok()?;
        seconds.trim().parse().ok().map(Timestamp::from_unix)
    }
}

impl ResultStore for GitCli {
    fn publish(&self, repo: &OnboardedRepo, publication: &Publication) -> Result<(), PublishError> {
        self.results.publish(repo, publication)
    }
}
