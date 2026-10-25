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

use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use crate::model::branch::branch_layout::BranchLayout;
use crate::model::branch::publication::Publication;
use crate::model::branch::summary_toml::SummaryToml;
use crate::model::commit_hash::CommitHash;
use crate::model::mutant_key::MutantKey;
use crate::model::roster::OnboardedRepo;
use crate::model::skip_reason::SkipReason;
use crate::traits::{
    Command, CommandResult, CommandRunner, RepoSource, ResultStore, Snapshot, SyncError,
};

/// Deadline for commands that talk to the remote, and for creating a checkout.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(5 * 60);
/// Deadline for other git commands.
pub const LOCAL_TIMEOUT: Duration = Duration::from_secs(60);

const MUTANTS_BRANCH: &str = "mutants";
const MUTANTS_REMOTE_REF: &str = "refs/remotes/origin/mutants";
const IDENTITY_NAME: &str = "sentinel4rust";
const IDENTITY_EMAIL: &str = "sentinel4rust@localhost";

/// git messages meaning the repository is gone or refuses the key.
const UNREACHABLE_MARKERS: &[&str] = &[
    "repository not found",
    "permission denied",
    "does not appear to be a git repository",
    "access denied",
    "could not read username",
    "authentication failed",
];

/// git, run through a [`CommandRunner`].
pub struct GitCli {
    runner: Rc<dyn CommandRunner>,
    work_dir: PathBuf,
}

impl GitCli {
    /// A git client keeping its clones under `work_dir`.
    pub fn new(runner: Rc<dyn CommandRunner>, work_dir: PathBuf) -> Self {
        Self { runner, work_dir }
    }

    /// The bare clone of `repo`.
    pub fn repo_dir(&self, repo: &OnboardedRepo) -> PathBuf {
        self.work_dir
            .join("repos")
            .join(format!("{}.git", repo.name))
    }

    fn run_dir(&self, repo: &OnboardedRepo) -> PathBuf {
        self.work_dir.join("runs").join(&repo.name)
    }

    fn stage_dir(&self, repo: &OnboardedRepo) -> PathBuf {
        self.work_dir.join("stage").join(&repo.name)
    }

    fn command(dir: &Path, args: &[&str], timeout: Duration) -> Command {
        Command::new("git", args, dir, timeout)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
    }

    fn run(&self, command: &Command) -> Result<String, CommandResult> {
        let result = self.runner.run(command);
        match result.success() {
            Some(stdout) => Ok(stdout.to_string()),
            None => Err(result),
        }
    }

    fn git(&self, dir: &Path, args: &[&str], timeout: Duration) -> Result<String, CommandResult> {
        self.run(&Self::command(dir, args, timeout))
    }

    fn local(&self, dir: &Path, args: &[&str]) -> Result<String, String> {
        self.git(dir, args, LOCAL_TIMEOUT)
            .map_err(|result| format!("git {}: {}", args.join(" "), result.describe()))
    }

    /// Removes leftovers of a crash: run checkouts, staging areas, stale worktree entries.
    pub fn prune(&self, repos: &[OnboardedRepo]) {
        for sub in ["runs", "stage"] {
            let _ = fs::remove_dir_all(self.work_dir.join(sub));
        }
        for repo in repos {
            let dir = self.repo_dir(repo);
            if dir.join("HEAD").exists() {
                let _ = self.local(&dir, &["worktree", "prune"]);
            }
        }
    }

    fn ensure_clone(&self, repo: &OnboardedRepo) -> Result<PathBuf, String> {
        let dir = self.repo_dir(repo);
        if dir.join("HEAD").exists() {
            let current = self.local(&dir, &["remote", "get-url", "origin"])?;
            if current.trim() != repo.url {
                self.local(&dir, &["remote", "set-url", "origin", &repo.url])?;
            }
            return Ok(dir);
        }
        let parent = dir.parent().unwrap_or(&self.work_dir).to_path_buf();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        let dir_text = dir.to_string_lossy().into_owned();
        self.local(&parent, &["init", "--quiet", "--bare", &dir_text])?;
        self.local(&dir, &["remote", "add", "origin", &repo.url])?;
        Ok(dir)
    }

    fn classify(result: &CommandResult, what: &str) -> SyncError {
        let message = format!("{what}: {}", result.describe());
        let text = match result {
            CommandResult::Exited { stderr, .. } => stderr.to_lowercase(),
            _ => String::new(),
        };
        if UNREACHABLE_MARKERS.iter().any(|m| text.contains(m)) {
            SyncError::Unreachable(message)
        } else {
            SyncError::Transient(message)
        }
    }

    fn show(&self, dir: &Path, object: &str) -> Option<String> {
        self.git(dir, &["show", object], LOCAL_TIMEOUT).ok()
    }

    fn exists_at(&self, dir: &Path, head: &CommitHash, path: &str) -> bool {
        let object = format!("{head}:{path}");
        self.git(dir, &["cat-file", "-e", &object], LOCAL_TIMEOUT)
            .is_ok()
    }

    fn readiness(&self, dir: &Path, head: &CommitHash) -> Result<(), SkipReason> {
        if !self.exists_at(dir, head, "Cargo.toml") {
            Err(SkipReason::NotACrate)
        } else if !self.exists_at(dir, head, ".cargo/mutants.toml") {
            Err(SkipReason::NoFilters)
        } else {
            Ok(())
        }
    }

    fn write_commit(
        &self,
        dir: &Path,
        stage: &Path,
        publication: &Publication,
    ) -> Result<String, String> {
        let index = stage.join("index");
        let index_text = index.to_string_lossy().into_owned();
        for (number, file_entry) in BranchLayout.files(publication).iter().enumerate() {
            let file = stage.join(format!("blob-{number}"));
            fs::write(&file, &file_entry.contents)
                .map_err(|e| format!("write {}: {e}", file.display()))?;
            let blob = self.local(dir, &["hash-object", "-w", &file.to_string_lossy()])?;
            let info = format!("100644,{},{}", blob.trim(), file_entry.path);
            let command = Self::command(
                dir,
                &["update-index", "--add", "--cacheinfo", &info],
                LOCAL_TIMEOUT,
            )
            .env("GIT_INDEX_FILE", &index_text);
            self.run(&command)
                .map_err(|r| format!("git update-index: {}", r.describe()))?;
        }
        let tree = self
            .run(
                &Self::command(dir, &["write-tree"], LOCAL_TIMEOUT)
                    .env("GIT_INDEX_FILE", &index_text),
            )
            .map_err(|r| format!("git write-tree: {}", r.describe()))?;
        let parent = self
            .git(
                dir,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("{MUTANTS_REMOTE_REF}^{{commit}}"),
                ],
                LOCAL_TIMEOUT,
            )
            .ok();
        let message = BranchLayout.message(publication);
        let mut args = vec!["commit-tree", tree.trim()];
        if let Some(parent) = &parent {
            args.extend(["-p", parent.trim()]);
        }
        args.extend(["-m", &message]);
        let command = Self::command(dir, &args, LOCAL_TIMEOUT)
            .env("GIT_AUTHOR_NAME", IDENTITY_NAME)
            .env("GIT_AUTHOR_EMAIL", IDENTITY_EMAIL)
            .env("GIT_COMMITTER_NAME", IDENTITY_NAME)
            .env("GIT_COMMITTER_EMAIL", IDENTITY_EMAIL);
        self.run(&command)
            .map(|commit| commit.trim().to_string())
            .map_err(|r| format!("git commit-tree: {}", r.describe()))
    }
}

impl RepoSource for GitCli {
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError> {
        let dir = self.ensure_clone(repo).map_err(SyncError::Transient)?;
        let branch_ref = format!("refs/heads/{}", repo.branch);
        let mutants_ref = format!("refs/heads/{MUTANTS_BRANCH}");
        let listing = self
            .git(
                &dir,
                &["ls-remote", "--heads", "origin", &branch_ref, &mutants_ref],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| Self::classify(&r, "git ls-remote"))?;
        let refs: Vec<&str> = listing
            .lines()
            .filter_map(|line| line.split_once('\t').map(|(_, name)| name.trim()))
            .collect();
        if !refs.contains(&branch_ref.as_str()) {
            return Err(SyncError::NoBranch);
        }
        let has_mutants = refs.contains(&mutants_ref.as_str());
        let branch_spec = format!("+{branch_ref}:refs/remotes/origin/{}", repo.branch);
        let mutants_spec = format!("+{mutants_ref}:{MUTANTS_REMOTE_REF}");
        let mut args = vec!["fetch", "--quiet", "--no-tags", "origin", &branch_spec];
        if has_mutants {
            args.push(&mutants_spec);
        }
        self.git(&dir, &args, NETWORK_TIMEOUT)
            .map_err(|r| Self::classify(&r, "git fetch"))?;
        if !has_mutants {
            let _ = self.git(
                &dir,
                &["update-ref", "-d", MUTANTS_REMOTE_REF],
                LOCAL_TIMEOUT,
            );
        }
        let head_text = self
            .local(
                &dir,
                &[
                    "rev-parse",
                    "--verify",
                    &format!("refs/remotes/origin/{}^{{commit}}", repo.branch),
                ],
            )
            .map_err(SyncError::Transient)?;
        let head = CommitHash::parse(&head_text).ok_or_else(|| {
            SyncError::Transient(format!("bad commit hash `{}`", head_text.trim()))
        })?;
        let (record, previous_missed) = if has_mutants {
            let record = self
                .show(&dir, &format!("{MUTANTS_REMOTE_REF}:summary.toml"))
                .and_then(|text| SummaryToml.parse(&text).ok());
            let missed = self
                .show(&dir, &format!("{MUTANTS_REMOTE_REF}:missed.txt"))
                .map(|text| {
                    text.lines()
                        .filter(|l| !l.trim().is_empty())
                        .map(MutantKey::new)
                        .collect()
                })
                .unwrap_or_default();
            (record, missed)
        } else {
            (None, Vec::new())
        };
        let readiness = self.readiness(&dir, &head);
        Ok(Snapshot {
            head,
            record,
            previous_missed,
            readiness,
        })
    }

    fn checkout(&self, repo: &OnboardedRepo, head: &CommitHash) -> Result<PathBuf, String> {
        self.remove_checkout(repo);
        let dir = self.repo_dir(repo);
        let run_dir = self.run_dir(repo);
        if let Some(parent) = run_dir.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        let run_text = run_dir.to_string_lossy().into_owned();
        self.git(
            &dir,
            &[
                "worktree",
                "add",
                "--quiet",
                "--detach",
                "--force",
                &run_text,
                head.as_str(),
            ],
            NETWORK_TIMEOUT,
        )
        .map_err(|r| format!("git worktree add: {}", r.describe()))?;
        Ok(run_dir)
    }

    fn remove_checkout(&self, repo: &OnboardedRepo) {
        let dir = self.repo_dir(repo);
        let run_dir = self.run_dir(repo);
        if run_dir.exists() && dir.join("HEAD").exists() {
            let run_text = run_dir.to_string_lossy().into_owned();
            let _ = self.local(
                &dir,
                &["worktree", "remove", "--force", "--force", &run_text],
            );
        }
        let _ = fs::remove_dir_all(&run_dir);
        if dir.join("HEAD").exists() {
            let _ = self.local(&dir, &["worktree", "prune"]);
        }
    }
}

impl ResultStore for GitCli {
    fn publish(&self, repo: &OnboardedRepo, publication: &Publication) -> Result<(), String> {
        let dir = self.repo_dir(repo);
        let stage = self.stage_dir(repo);
        let _ = fs::remove_dir_all(&stage);
        fs::create_dir_all(&stage).map_err(|e| format!("create {}: {e}", stage.display()))?;
        let result = self
            .write_commit(&dir, &stage, publication)
            .and_then(|commit| {
                let refspec = format!("{commit}:refs/heads/{MUTANTS_BRANCH}");
                self.git(
                    &dir,
                    &["push", "--quiet", "origin", &refspec],
                    NETWORK_TIMEOUT,
                )
                .map_err(|r| format!("git push: {}", r.describe()))?;
                self.local(&dir, &["update-ref", MUTANTS_REMOTE_REF, &commit])?;
                Ok(())
            });
        let _ = fs::remove_dir_all(&stage);
        result
    }
}
