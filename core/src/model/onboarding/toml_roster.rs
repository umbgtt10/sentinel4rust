// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The roster file, read and validated (`deploy/roster.example.toml`).

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::time::Duration;

use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::roster::Roster;
use crate::model::onboarding::roster_error::RosterError;
use crate::model::toml::table::Table;
use crate::model::toml::toml_parser::TomlParser;

const ROOT_KEYS: &[&str] = &[
    "work_dir",
    "pause_seconds",
    "jobs",
    "run_timeout_hours",
    "config_version",
    "stale_after_hours",
    "run_unit",
    "results_url",
];
const REPO_KEYS: &[&str] = &["name", "url", "branch", "jobs", "run_timeout_hours"];

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
        let default_hours =
            Self::positive(root.integer("run_timeout_hours")?, 12, "run_timeout_hours")?;
        let mut names = BTreeSet::new();
        let mut repos = Vec::new();
        for table in doc.arrays.get("repo").into_iter().flatten() {
            let repo = Self::repo(table, &names, default_jobs, default_hours)?;
            names.insert(repo.name.clone());
            repos.push(repo);
        }
        if repos.is_empty() {
            return Err(RosterError("no [[repo]] entries".to_string()));
        }
        let work_dir = root.required_string("work_dir")?;
        let results_url = root.required_string("results_url")?;
        if results_url.trim().is_empty() {
            return Err(RosterError("empty `results_url`".to_string()));
        }
        Ok(Roster {
            work_dir,
            pause: Duration::from_secs(Self::positive(
                root.integer("pause_seconds")?,
                300,
                "pause_seconds",
            )?),
            jobs: default_jobs,
            run_timeout: Self::hours(default_hours),
            config_version: root.integer("config_version")?.unwrap_or(1),
            stale_after: Self::hours(Self::positive(
                root.integer("stale_after_hours")?,
                24,
                "stale_after_hours",
            )?),
            results_url,
            run_unit: Self::run_unit(root.string("run_unit")?)?,
            repos,
        })
    }

    fn hours(hours: u64) -> Duration {
        Duration::from_secs(hours.saturating_mul(3600))
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

    fn repo(
        table: &Table,
        names: &BTreeSet<String>,
        default_jobs: u32,
        default_hours: u64,
    ) -> Result<OnboardedRepo, RosterError> {
        table.only(REPO_KEYS)?;
        let line = table.line();
        let name = table.required_string("name")?;
        if !Self::valid_name(&name) {
            return Err(RosterError(format!(
                "line {line}: invalid name `{name}` (letters, digits, `-`, `_`, `.`)"
            )));
        }
        if names.contains(&name) {
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
        Ok(OnboardedRepo {
            name,
            url,
            branch,
            jobs: Self::jobs(table.integer("jobs")?, default_jobs, line)?,
            run_timeout: Self::hours(Self::positive(
                table.integer("run_timeout_hours")?,
                i64::try_from(default_hours).unwrap_or(i64::MAX),
                "run_timeout_hours",
            )?),
        })
    }

    fn run_unit(unit: Option<String>) -> Result<Option<String>, RosterError> {
        match unit {
            Some(unit)
                if unit.is_empty()
                    || !unit
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) =>
            {
                Err(RosterError(format!(
                    "invalid `run_unit` `{unit}` (letters, digits, `-`, `_`)"
                )))
            }
            unit => Ok(unit),
        }
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
