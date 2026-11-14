// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! GitHub's API, with the installation token.

use std::rc::Rc;

use crate::system::github::curl::Curl;
use crate::system::github::secret_dir::SecretDir;
use crate::system::github::token_source::TokenSource;
use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust::traits::command_runner::CommandRunner;
use std::path::PathBuf;

/// GitHub's API, read with the token a [`TokenSource`] answers.
pub struct GithubApi {
    curl: Curl,
    token: Rc<dyn TokenSource>,
    secrets: SecretDir,
}

impl GithubApi {
    /// The API through `runner`, with `token`, its header written under `secrets`.
    pub fn new(
        runner: Rc<dyn CommandRunner>,
        token: Rc<dyn TokenSource>,
        secrets: SecretDir,
    ) -> Self {
        Self {
            curl: Curl::new(runner),
            token,
            secrets,
        }
    }

    /// Whether `url` answers 200 rather than 404; anything else is a failure.
    pub fn exists(&self, url: &str) -> Result<bool, DiscoveryError> {
        let status = self.curl.status(&self.header()?, url)?;
        match status.trim() {
            "200" => Ok(true),
            "404" => Ok(false),
            other => Err(DiscoveryError(format!("{url}: HTTP {other}"))),
        }
    }

    /// The body of a GET of `url`.
    pub fn get(&self, url: &str) -> Result<String, DiscoveryError> {
        self.curl.get(&self.header()?, url)
    }

    fn header(&self) -> Result<PathBuf, DiscoveryError> {
        let token = self.token.token()?;
        self.secrets.write(
            "api.header",
            format!("Authorization: Bearer {token}\n").as_bytes(),
        )
    }
}
