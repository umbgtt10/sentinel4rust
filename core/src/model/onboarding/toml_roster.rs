// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The roster file, read and validated (`deploy/roster.example.toml`).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::roster::Roster;
use crate::model::onboarding::roster_error::RosterError;
use crate::model::toml::toml_parser::TomlParser;

const ROOT_KEYS: &[&str] = &[
    "work_dir",
    "pause_seconds",
    "jobs",
    "run_timeout_hours",
    "config_version",
];
const REPO_KEYS: &[&str] = &["name", "url", "branch", "jobs"];

/// The roster file, read and validated.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlRoster;

impl TomlRoster {
    /// Parses and validates the roster file's text.
    pub fn parse(&self, text: &str) -> Result<Roster, RosterError> {
        let doc = TomlParser.parse(text)?;
        let root = &doc.root;
        root.only(ROOT_KEYS)?;
        if let Some(name) = doc.arrays.keys().find(|name| *name != "repo") {
            return Err(RosterError(format!("unknown section `[[{name}]]`")));
        }
        let default_jobs = Self::jobs(root.integer("jobs")?, 1, 1)?;
        let mut names = BTreeSet::new();
        let mut repos = Vec::new();
        for table in doc.arrays.get("repo").into_iter().flatten() {
            table.only(REPO_KEYS)?;
            let line = table.line();
            let name = table.required_string("name")?;
            if !Self::valid_name(&name) {
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
            if !Self::valid_branch(&branch) || branch == "mutants" {
                return Err(RosterError(format!(
                    "line {line}: invalid branch `{branch}`"
                )));
            }
            repos.push(OnboardedRepo {
                name,
                url,
                branch,
                jobs: Self::jobs(table.integer("jobs")?, default_jobs, line)?,
            });
        }
        if repos.is_empty() {
            return Err(RosterError("no [[repo]] entries".to_string()));
        }
        Ok(Roster {
            work_dir: PathBuf::from(root.required_string("work_dir")?),
            pause: Duration::from_secs(Self::positive(
                root.integer("pause_seconds")?,
                300,
                "pause_seconds",
            )?),
            jobs: default_jobs,
            run_timeout: Duration::from_secs(
                Self::positive(root.integer("run_timeout_hours")?, 12, "run_timeout_hours")? * 3600,
            ),
            config_version: root.integer("config_version")?.unwrap_or(1),
            repos,
        })
    }

    fn jobs(value: Option<i64>, default: u32, line: usize) -> Result<u32, RosterError> {
        let value = Self::positive(value, i64::from(default), "jobs")?;
        u32::try_from(value).map_err(|_| RosterError(format!("line {line}: `jobs` is too large")))
    }

    fn positive(value: Option<i64>, default: i64, what: &str) -> Result<u64, RosterError> {
        let value = value.unwrap_or(default);
        u64::try_from(value)
            .ok()
            .filter(|v| *v > 0)
            .ok_or_else(|| RosterError(format!("`{what}` must be positive, got {value}")))
    }

    fn valid_branch(branch: &str) -> bool {
        !branch.is_empty()
            && !branch.starts_with('-')
            && !branch.contains("..")
            && branch
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
    }

    fn valid_name(name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('.')
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }
}
