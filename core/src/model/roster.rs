// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The roster: which repositories to watch, and how (design 3).

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crate::toml;

/// Global settings and the onboarded repositories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roster {
    /// Clones, worktrees and cargo-mutants output live here.
    pub work_dir: PathBuf,
    /// Polling interval per repository.
    pub pause: Duration,
    /// Default cargo-mutants `--jobs`.
    pub jobs: u32,
    /// Cap on one cargo-mutants run.
    pub run_timeout: Duration,
    /// Bump to rerun every repository.
    pub config_version: i64,
    /// The repositories, in roster order.
    pub repos: Vec<OnboardedRepo>,
}

/// One onboarded repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardedRepo {
    /// Unique; names the local clone.
    pub name: String,
    /// What git fetches from and pushes to.
    pub url: String,
    /// The branch tested; `main` by default.
    pub branch: String,
    /// cargo-mutants `--jobs` for this repository.
    pub jobs: u32,
}

/// A roster that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterError(pub String);

impl fmt::Display for RosterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "roster: {}", self.0)
    }
}

impl std::error::Error for RosterError {}

impl From<toml::TomlError> for RosterError {
    fn from(error: toml::TomlError) -> Self {
        Self(error.to_string())
    }
}

const ROOT_KEYS: &[&str] = &[
    "work_dir",
    "pause_seconds",
    "jobs",
    "run_timeout_hours",
    "config_version",
];
const REPO_KEYS: &[&str] = &["name", "url", "branch", "jobs"];

fn positive(value: Option<i64>, default: i64, what: &str) -> Result<u64, RosterError> {
    let value = value.unwrap_or(default);
    u64::try_from(value)
        .ok()
        .filter(|v| *v > 0)
        .ok_or_else(|| RosterError(format!("`{what}` must be positive, got {value}")))
}

fn jobs(value: Option<i64>, default: u32, line: usize) -> Result<u32, RosterError> {
    let value = positive(value, i64::from(default), "jobs")?;
    u32::try_from(value).map_err(|_| RosterError(format!("line {line}: `jobs` is too large")))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn valid_branch(branch: &str) -> bool {
    !branch.is_empty()
        && !branch.starts_with('-')
        && !branch.contains("..")
        && branch
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
}

impl Roster {
    /// Parses and validates the roster file's text.
    pub fn parse(text: &str) -> Result<Self, RosterError> {
        let doc = toml::parse(text)?;
        let root = &doc.root;
        root.only(ROOT_KEYS)?;
        if let Some(name) = doc.arrays.keys().find(|name| *name != "repo") {
            return Err(RosterError(format!("unknown section `[[{name}]]`")));
        }
        let default_jobs = jobs(root.integer("jobs")?, 1, 1)?;
        let mut names = BTreeSet::new();
        let mut repos = Vec::new();
        for table in doc.arrays.get("repo").into_iter().flatten() {
            table.only(REPO_KEYS)?;
            let line = table.line();
            let name = table.required_string("name")?;
            if !valid_name(&name) {
                return Err(RosterError(format!(
                    "line {line}: invalid name `{name}` (letters, digits, `-`, `_`, `.`)"
                )));
            }
            if !names.insert(name.clone()) {
                return Err(RosterError(format!("line {line}: duplicate name `{name}`")));
            }
            let url = table.required_string("url")?;
            if url.trim().is_empty() {
                return Err(RosterError(format!("line {line}: empty url")));
            }
            let branch = table
                .string("branch")?
                .unwrap_or_else(|| "main".to_string());
            if !valid_branch(&branch) || branch == "mutants" {
                return Err(RosterError(format!(
                    "line {line}: invalid branch `{branch}`"
                )));
            }
            repos.push(OnboardedRepo {
                name,
                url,
                branch,
                jobs: jobs(table.integer("jobs")?, default_jobs, line)?,
            });
        }
        if repos.is_empty() {
            return Err(RosterError("no [[repo]] entries".to_string()));
        }
        Ok(Self {
            work_dir: PathBuf::from(root.required_string("work_dir")?),
            pause: Duration::from_secs(positive(
                root.integer("pause_seconds")?,
                300,
                "pause_seconds",
            )?),
            jobs: default_jobs,
            run_timeout: Duration::from_secs(
                positive(root.integer("run_timeout_hours")?, 12, "run_timeout_hours")? * 3600,
            ),
            config_version: root.integer("config_version")?.unwrap_or(1),
            repos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = r#"
        work_dir = "/srv/s"
        pause_seconds = 60
        jobs = 6
        run_timeout_hours = 2
        config_version = 3

        [[repo]]
        name = "twin4rust"
        url = "git@github-twin4rust:umbgtt10/twin4rust.git"

        [[repo]]
        name = "etheram-raft"
        url = "git@github-etheram-raft:umbgtt10/etheram-raft.git"
        branch = "trunk"
        jobs = 2
    "#;

    #[test]
    fn parses_settings_and_repos_in_order() {
        let roster = Roster::parse(FULL).unwrap();
        assert_eq!(roster.work_dir, PathBuf::from("/srv/s"));
        assert_eq!(roster.pause, Duration::from_secs(60));
        assert_eq!(roster.run_timeout, Duration::from_secs(7200));
        assert_eq!(roster.config_version, 3);
        let names: Vec<&str> = roster.repos.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["twin4rust", "etheram-raft"]);
        assert_eq!(roster.repos[0].branch, "main");
        assert_eq!(roster.repos[0].jobs, 6);
        assert_eq!(roster.repos[1].branch, "trunk");
        assert_eq!(roster.repos[1].jobs, 2);
    }

    #[test]
    fn defaults_apply() {
        let roster =
            Roster::parse("work_dir = \"/w\"\n[[repo]]\nname = \"a\"\nurl = \"u\"").unwrap();
        assert_eq!(roster.pause, Duration::from_secs(300));
        assert_eq!(roster.jobs, 1);
        assert_eq!(roster.run_timeout, Duration::from_secs(12 * 3600));
        assert_eq!(roster.config_version, 1);
    }

    #[test]
    fn rejects_invalid_rosters() {
        let repo = "[[repo]]\nname = \"a\"\nurl = \"u\"\n";
        let cases = [
            (repo.to_string(), "work_dir"),
            ("work_dir = \"/w\"".to_string(), "no [[repo]]"),
            (
                format!("work_dir = \"/w\"\ntypo = 1\n{repo}"),
                "unknown key",
            ),
            (format!("work_dir = \"/w\"\n{repo}{repo}"), "duplicate name"),
            (format!("work_dir = \"/w\"\njobs = 0\n{repo}"), "positive"),
            (
                format!("work_dir = \"/w\"\n{repo}branch = \"mutants\""),
                "invalid branch",
            ),
            (
                "work_dir = \"/w\"\n[[repo]]\nname = \"../x\"\nurl = \"u\"".to_string(),
                "invalid name",
            ),
            (
                "work_dir = \"/w\"\n[[repo]]\nname = \"a\"\nurl = \" \"".to_string(),
                "empty url",
            ),
            (
                format!("work_dir = \"/w\"\n{repo}[[other]]\nx = 1"),
                "unknown section",
            ),
        ];
        for (text, fragment) in cases {
            let err = Roster::parse(&text).unwrap_err().0;
            assert!(err.contains(fragment), "{fragment}: {err}");
        }
    }
}
