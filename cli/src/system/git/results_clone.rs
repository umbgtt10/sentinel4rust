// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The results repository's bare clone: where every record is read and every publication is pushed
//! (`ADR-SharedResultsRepository`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::system::git::bare_clone::BareClone;
use crate::system::git::git_constants::{NETWORK_TIMEOUT, RESULTS_REMOTE_REF};
use crate::system::git::git_runner::GitRunner;
use crate::system::git::plumbing_commit::PlumbingCommit;
use crate::system::git::record_reader::RecordReader;
use crate::system::work_dir::WorkDir;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::publish_error::PublishError;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::protocol::git::remote_listing::RemoteListing;
use sentinel4rust::protocol::git::unreachable_classifier::UnreachableClassifier;

/// The results repository's one branch, the only ref ever pushed (`ADR-SharedResultsRepository`).
const RESULTS_BRANCH: &str = "main";

/// One bare clone of the results repository, its one branch fetched and pushed, one folder per watched
/// repository.
#[derive(Clone)]
pub struct ResultsClone {
    git: GitRunner,
    work_dir: WorkDir,
    url: String,
    clone: BareClone,
    records: RecordReader,
    commit: PlumbingCommit,
}

impl ResultsClone {
    /// The results repository at `url`, cloned under `work_dir` through `git`.
    pub fn new(git: GitRunner, work_dir: WorkDir, url: String) -> Self {
        Self {
            clone: BareClone::new(git.clone(), work_dir.clone()),
            records: RecordReader::new(git.clone()),
            commit: PlumbingCommit::new(git.clone()),
            git,
            work_dir,
            url,
        }
    }

    /// Fetches the results branch; drops the local copy when the repository has none yet.
    pub fn fetch(&self) -> Result<(), SyncError> {
        let dir = self
            .clone
            .ensure_at(&self.work_dir.results_clone(), &self.url)?;
        let branch_ref = format!("refs/heads/{RESULTS_BRANCH}");
        let listing = self
            .git
            .git(
                &dir,
                &["ls-remote", "--heads", "origin", &branch_ref],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| UnreachableClassifier.classify(&r, "git ls-remote"))?;
        if !RemoteListing::parse(&listing).has(&branch_ref) {
            let _ = self
                .git
                .local(&dir, &["update-ref", "-d", RESULTS_REMOTE_REF]);
            return Ok(());
        }
        let spec = format!("+{branch_ref}:{RESULTS_REMOTE_REF}");
        self.git
            .git(
                &dir,
                &["fetch", "--quiet", "--no-tags", "origin", &spec],
                NETWORK_TIMEOUT,
            )
            .map(|_| ())
            .map_err(|r| UnreachableClassifier.classify(&r, "git fetch"))
    }

    /// The survivors `repo`'s folder lists, as last fetched.
    pub fn previous_missed(&self, repo: &OnboardedRepo) -> Vec<MutantKey> {
        self.records.previous_missed(&self.dir(), &repo.name)
    }

    /// Writes `publication` as a commit replacing `repo`'s folder on the fetched branch, and pushes it; rejected
    /// when the branch moved since the fetch.
    pub fn publish(
        &self,
        repo: &OnboardedRepo,
        publication: &Publication,
    ) -> Result<(), PublishError> {
        let dir = self.dir();
        let stage = self.work_dir.stage(repo);
        let _ = fs::remove_dir_all(&stage);
        fs::create_dir_all(&stage)
            .map_err(|e| PublishError(format!("create {}: {e}", stage.display())))?;
        let result = self
            .commit
            .write(&dir, &stage, &repo.name, publication)
            .and_then(|commit| self.push(&dir, &commit));
        let _ = fs::remove_dir_all(&stage);
        result
    }

    /// What `repo`'s `summary.toml` holds, as last fetched.
    pub fn record(&self, repo: &OnboardedRepo) -> RecordState {
        self.records.record(&self.dir(), &repo.name)
    }

    fn dir(&self) -> PathBuf {
        self.work_dir.results_clone()
    }

    fn push(&self, dir: &Path, commit: &str) -> Result<(), PublishError> {
        let refspec = format!("{commit}:refs/heads/{RESULTS_BRANCH}");
        self.git
            .git(
                dir,
                &["push", "--quiet", "origin", &refspec],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| PublishError(format!("git push: {}", r.describe())))?;
        self.git
            .local(dir, &["update-ref", RESULTS_REMOTE_REF, commit])
            .map_err(|failure| PublishError(failure.to_string()))?;
        Ok(())
    }
}
