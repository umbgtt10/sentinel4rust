// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The roster's GitHub App keys, read and validated (`ADR-GithubAppDiscovery`).

use alloc::format;

use crate::model::onboarding::github_app::GithubApp;
use crate::model::onboarding::roster_error::RosterError;
use crate::model::toml::table::Table;

const APP_ID: &str = "github_app_id";
const INSTALLATION_ID: &str = "github_app_installation_id";
const KEY: &str = "github_app_key";

/// The roster's `github_app_*` keys: all three, or none.
#[derive(Debug, Clone, Copy, Default)]
pub struct GithubAppToml;

impl GithubAppToml {
    /// The App the roster's root names; `None` when it names none of the three keys.
    pub fn parse(&self, root: &Table) -> Result<Option<GithubApp>, RosterError> {
        if !root
            .keys()
            .any(|key| [APP_ID, INSTALLATION_ID, KEY].contains(&key))
        {
            return Ok(None);
        }
        Ok(Some(GithubApp {
            app_id: Self::id(root, APP_ID)?,
            installation_id: Self::id(root, INSTALLATION_ID)?,
            key: root.required_string(KEY)?,
        }))
    }

    fn id(root: &Table, key: &str) -> Result<u64, RosterError> {
        let value = root.required_integer(key)?;
        u64::try_from(value)
            .ok()
            .filter(|id| *id > 0)
            .ok_or_else(|| RosterError(format!("`{key}` must be positive, got {value}")))
    }
}
