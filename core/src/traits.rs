// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The contracts every part of the tool talks through (design 4). Each has a system implementation and, in
//! tests, a fake.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::model::branch::publication::Publication;
use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;
use crate::model::mutant_key::MutantKey;
use crate::model::mutation_run::MutationRun;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::skip_reason::SkipReason;
use crate::model::timestamp::Timestamp;

/// A program to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The executable.
    pub program: String,
    /// Its arguments.
    pub args: Vec<String>,
    /// The working directory.
    pub dir: PathBuf,
    /// Extra environment variables.
    pub env: Vec<(String, String)>,
    /// Killed, with its process group, after this long.
    pub timeout: Duration,
}

impl Command {
    /// A command with no extra environment.
    pub fn new(program: &str, args: &[&str], dir: &Path, timeout: Duration) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(ToString::to_string).collect(),
            dir: dir.to_path_buf(),
            env: Vec::new(),
            timeout,
        }
    }

    /// Adds an environment variable.
    #[must_use]
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    /// The command line, for logs.
    pub fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// How a command ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// It exited; `code` is `None` when killed by a signal.
    Exited {
        /// The exit code.
        code: Option<i32>,
        /// Standard output.
        stdout: String,
        /// Standard error.
        stderr: String,
    },
    /// It ran past its deadline and was killed.
    TimedOut {
        /// Standard output up to the kill.
        stdout: String,
        /// Standard error up to the kill.
        stderr: String,
    },
    /// It could not be started.
    FailedToStart(String),
}

impl CommandResult {
    /// Standard output if the command exited with 0.
    pub fn success(&self) -> Option<&str> {
        match self {
            Self::Exited {
                code: Some(0),
                stdout,
                ..
            } => Some(stdout),
            _ => None,
        }
    }

    /// A one-line description of a failure.
    pub fn describe(&self) -> String {
        match self {
            Self::Exited { code, stderr, .. } => {
                let code = code.map_or_else(|| "a signal".to_string(), |c| c.to_string());
                let first = stderr.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
                format!("exited with {code}: {}", first.trim())
            }
            Self::TimedOut { .. } => "timed out".to_string(),
            Self::FailedToStart(message) => format!("could not start: {message}"),
        }
    }
}

/// Runs programs.
pub trait CommandRunner {
    /// Runs `command` to completion or its deadline.
    fn run(&self, command: &Command) -> CommandResult;
}

/// The current time.
pub trait Clock {
    /// Now, UTC.
    fn now(&self) -> Timestamp;
}

/// Waiting.
pub trait Pause {
    /// Waits for `duration`.
    fn pause(&self, duration: Duration);
}

/// Where the one line per visit goes.
pub trait Journal {
    /// Writes one line.
    fn line(&self, text: &str);
}

/// What a fetch found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// The configured branch's commit.
    pub head: CommitHash,
    /// The record on `mutants`, if there is a usable one.
    pub record: Option<RunRecord>,
    /// `missed.txt` on `mutants`, for new/gone.
    pub previous_missed: Vec<MutantKey>,
    /// Whether `head` can be tested.
    pub readiness: Result<(), SkipReason>,
}

/// Why a fetch failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// Repository not found or permission denied.
    Unreachable(String),
    /// Anything else, likely temporary.
    Transient(String),
    /// The configured branch does not exist on the remote.
    NoBranch,
}

/// Reads repositories.
pub trait RepoSource {
    /// Clones if needed, fetches, and reads the branch's commit and the recorded result.
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError>;
    /// Creates a detached checkout of `head` that nobody works in; returns its path.
    fn checkout(&self, repo: &OnboardedRepo, head: &CommitHash) -> Result<PathBuf, String>;
    /// Removes the checkout, whatever state it is in.
    fn remove_checkout(&self, repo: &OnboardedRepo);
}

/// Runs mutation testing.
pub trait MutationRunner {
    /// Runs cargo-mutants in `checkout`, killed after `timeout`.
    fn run(
        &self,
        repo: &OnboardedRepo,
        checkout: &Path,
        jobs: u32,
        timeout: Duration,
    ) -> MutationRun;
}

/// Publishes results.
pub trait ResultStore {
    /// Writes `publication` as a new commit on `mutants` and pushes only that branch.
    fn publish(&self, repo: &OnboardedRepo, publication: &Publication) -> Result<(), String>;
}

macro_rules! forward_through_rc {
    ($($trait:ident { $(fn $name:ident(&self $(, $arg:ident: $ty:ty)*) -> $ret:ty;)* })*) => {$(
        impl<T: $trait + ?Sized> $trait for std::rc::Rc<T> {
            $(fn $name(&self $(, $arg: $ty)*) -> $ret {
                (**self).$name($($arg),*)
            })*
        }
    )*};
}

// Lets one shared implementation (for instance `Rc<GitCli>`, or a fake in tests) fill several roles.
forward_through_rc! {
    CommandRunner { fn run(&self, command: &Command) -> CommandResult; }
    Clock { fn now(&self) -> Timestamp; }
    Pause { fn pause(&self, duration: Duration) -> (); }
    Journal { fn line(&self, text: &str) -> (); }
    RepoSource {
        fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError>;
        fn checkout(&self, repo: &OnboardedRepo, head: &CommitHash) -> Result<PathBuf, String>;
        fn remove_checkout(&self, repo: &OnboardedRepo) -> ();
    }
    MutationRunner {
        fn run(&self, repo: &OnboardedRepo, checkout: &Path, jobs: u32, timeout: Duration) -> MutationRun;
    }
    ResultStore {
        fn publish(&self, repo: &OnboardedRepo, publication: &Publication) -> Result<(), String>;
    }
}
