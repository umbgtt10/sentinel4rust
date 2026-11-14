// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the roster's GitHub App gives the composition root (`ADR-GithubAppDiscovery`): git that carries its token,
//! and discovery through it, sharing one token.

use std::path::Path;
use std::rc::Rc;

use crate::system::github::app_token::AppToken;
use crate::system::github::git_token_runner::GitTokenRunner;
use crate::system::github::github_api::GithubApi;
use crate::system::github::github_app_discovery::GithubAppDiscovery;
use crate::system::github::secret_dir::SecretDir;
use sentinel4rust::model::onboarding::github_app::GithubApp;
use sentinel4rust::traits::clock::Clock;
use sentinel4rust::traits::command_runner::CommandRunner;
use sentinel4rust::traits::repo_discovery::RepoDiscovery;

/// The directory beside the App's key its token's files go in.
const SECRETS: &str = "sentinel4rust-github";

/// The GitHub App's token, and what is built on it.
pub struct GithubAccess {
    runner: Rc<dyn CommandRunner>,
    token: Rc<AppToken>,
    secrets: SecretDir,
}

impl GithubAccess {
    /// For `app`, through `runner`, on `clock`; its files beside its key, where the run unit cannot read them.
    pub fn new(runner: Rc<dyn CommandRunner>, clock: Rc<dyn Clock>, app: &GithubApp) -> Self {
        let secrets = SecretDir::new(
            Path::new(&app.key)
                .parent()
                .unwrap_or(Path::new("/"))
                .join(SECRETS),
        );
        Self {
            token: Rc::new(AppToken::new(
                runner.clone(),
                clock,
                app.clone(),
                secrets.clone(),
            )),
            runner,
            secrets,
        }
    }

    /// Discovery over the App's installation.
    pub fn discovery(&self) -> Rc<dyn RepoDiscovery> {
        Rc::new(GithubAppDiscovery::new(GithubApi::new(
            self.runner.clone(),
            self.token.clone(),
            self.secrets.clone(),
        )))
    }

    /// The runner for git: its network commands carry the token.
    pub fn git_runner(&self) -> Rc<dyn CommandRunner> {
        Rc::new(GitTokenRunner::new(self.runner.clone(), self.token.clone()))
    }
}
