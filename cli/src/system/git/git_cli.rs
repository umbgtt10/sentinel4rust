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
use crate::system::git::git_constants::{LOCAL_TIMEOUT, MUTANTS_REMOTE_REF, NETWORK_TIMEOUT};
use crate::system::git::git_runner::GitRunner;
use crate::system::git::plumbing_commit::PlumbingCommit;
use crate::system::git::record_reader::RecordReader;
use crate::system::work_dir::WorkDir;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::branch::record_state::RecordState;
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

const MUTANTS_BRANCH: &str = "mutants";

/// git, run through a [`CommandRunner`]: fetches, checks out and publishes, and only ever pushes
/// `refs/heads/mutants`.
pub struct GitCli {
    git: GitRunner,
    work_dir: WorkDir,
    clone: BareClone,
    readiness: CrateReadiness,
    records: RecordReader,
    worktree: DetachedWorktree,
    commit: PlumbingCommit,
}

impl GitCli {
    /// A git client keeping its clones, checkouts and staging areas under `work_dir`.
    pub fn new(runner: Rc<dyn CommandRunner>, work_dir: WorkDir) -> Self {
        let git = GitRunner::new(runner);
        Self {
            clone: BareClone::new(git.clone(), work_dir.clone()),
            readiness: CrateReadiness::new(git.clone()),
            records: RecordReader::new(git.clone()),
            worktree: DetachedWorktree::new(git.clone(), work_dir.clone()),
            commit: PlumbingCommit::new(git.clone()),
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

    fn fetch(&self, dir: &Path, repo: &OnboardedRepo, has_mutants: bool) -> Result<(), SyncError> {
        let branch_spec = format!(
            "+refs/heads/{}:refs/remotes/origin/{}",
            repo.branch, repo.branch
        );
        let mutants_spec = format!("+refs/heads/{MUTANTS_BRANCH}:{MUTANTS_REMOTE_REF}");
        let mut args = vec!["fetch", "--quiet", "--no-tags", "origin", &branch_spec];
        if has_mutants {
            args.push(&mutants_spec);
        }
        self.git
            .git(dir, &args, NETWORK_TIMEOUT)
            .map_err(|r| UnreachableClassifier.classify(&r, "git fetch"))?;
        if !has_mutants {
            let _ = self.git.git(
                dir,
                &["update-ref", "-d", MUTANTS_REMOTE_REF],
                LOCAL_TIMEOUT,
            );
        }
        Ok(())
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

    fn push(&self, dir: &Path, commit: &str) -> Result<(), PublishError> {
        let refspec = format!("{commit}:refs/heads/{MUTANTS_BRANCH}");
        self.git
            .git(
                dir,
                &["push", "--quiet", "origin", &refspec],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| PublishError(format!("git push: {}", r.describe())))?;
        self.git
            .local(dir, &["update-ref", MUTANTS_REMOTE_REF, commit])
            .map_err(|failure| PublishError(failure.to_string()))?;
        Ok(())
    }
}

impl RepoSource for GitCli {
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError> {
        let dir = self.clone.ensure(repo)?;
        let branch_ref = format!("refs/heads/{}", repo.branch);
        let mutants_ref = format!("refs/heads/{MUTANTS_BRANCH}");
        let listing = self
            .git
            .git(
                &dir,
                &["ls-remote", "--heads", "origin", &branch_ref, &mutants_ref],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| UnreachableClassifier.classify(&r, "git ls-remote"))?;
        let listing = RemoteListing::parse(&listing);
        if !listing.has(&branch_ref) {
            return Err(SyncError::NoBranch);
        }
        let has_mutants = listing.has(&mutants_ref);
        self.fetch(&dir, repo, has_mutants)?;
        let head = self.head(&dir, repo)?;
        let (record, previous_missed) = if has_mutants {
            (
                self.records.record(&dir),
                self.records.previous_missed(&dir),
            )
        } else {
            (RecordState::Absent, Vec::new())
        };
        let readiness = self.readiness.check(&dir, &head);
        Ok(Snapshot {
            head,
            record,
            previous_missed,
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
        let dir = self.work_dir.bare_clone(repo);
        let stage = self.work_dir.stage(repo);
        let _ = fs::remove_dir_all(&stage);
        fs::create_dir_all(&stage)
            .map_err(|e| PublishError(format!("create {}: {e}", stage.display())))?;
        let result = self
            .commit
            .write(&dir, &stage, publication)
            .and_then(|commit| self.push(&dir, &commit));
        let _ = fs::remove_dir_all(&stage);
        result
    }
}
