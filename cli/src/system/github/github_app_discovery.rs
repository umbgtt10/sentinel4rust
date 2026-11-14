// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Discovery through the GitHub App: every repository it is installed on, not archived, with a
//! `.cargo/mutants.toml` on its default branch.

use sentinel4rust::model::github::discovered_repo::DiscoveredRepo;
use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust::traits::repo_discovery::RepoDiscovery;

use crate::system::github::github_api::GithubApi;
use serde_json::Value;
use serde_json::from_str;

/// Repositories listed per page.
const PER_PAGE: usize = 100;

/// Discovery over the App's installation.
pub struct GithubAppDiscovery {
    api: GithubApi,
}

impl GithubAppDiscovery {
    /// Discovery through `api`.
    pub fn new(api: GithubApi) -> Self {
        Self { api }
    }

    fn page(&self, number: usize) -> Result<Vec<Value>, DiscoveryError> {
        let url = format!(
            "https://api.github.com/installation/repositories?per_page={PER_PAGE}&page={number}"
        );
        let body = self.api.get(&url)?;
        let answer: Value = from_str(&body).map_err(|e| DiscoveryError(format!("{url}: {e}")))?;
        answer["repositories"]
            .as_array()
            .cloned()
            .ok_or_else(|| DiscoveryError(format!("{url}: no repositories in the answer")))
    }

    fn wanted(&self, repo: &Value) -> Result<Option<DiscoveredRepo>, DiscoveryError> {
        let (Some(name), Some(full_name), Some(url), Some(branch)) = (
            repo["name"].as_str(),
            repo["full_name"].as_str(),
            repo["clone_url"].as_str(),
            repo["default_branch"].as_str(),
        ) else {
            return Ok(None);
        };
        if repo["archived"].as_bool() == Some(true) {
            return Ok(None);
        }
        let filters = format!(
            "https://api.github.com/repos/{full_name}/contents/.cargo/mutants.toml?ref={branch}"
        );
        Ok(self.api.exists(&filters)?.then(|| DiscoveredRepo {
            name: name.to_string(),
            url: url.to_string(),
            branch: branch.to_string(),
        }))
    }
}

impl RepoDiscovery for GithubAppDiscovery {
    fn discover(&self) -> Result<Vec<DiscoveredRepo>, DiscoveryError> {
        let mut found = Vec::new();
        for number in 1.. {
            let page = self.page(number)?;
            for repo in &page {
                found.extend(self.wanted(repo)?);
            }
            if page.len() < PER_PAGE {
                break;
            }
        }
        Ok(found)
    }
}
