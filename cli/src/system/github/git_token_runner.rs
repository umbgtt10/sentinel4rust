// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! git over https to GitHub with the App's installation token, carried in the environment -- which only the
//! sentinel's own processes can read -- and never on the command line or in a repository's configuration.

use std::rc::Rc;

use crate::system::github::token_source::TokenSource;
use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::model::github::base64::Base64;
use sentinel4rust::traits::command_runner::CommandRunner;

/// The git subcommands that talk to a remote.
const NETWORK: &[&str] = &["clone", "fetch", "ls-remote", "push"];

/// A [`CommandRunner`] that gives git's network commands the token, as an extra HTTP header for github.com, and
/// runs every other command as it is.
pub struct GitTokenRunner {
    inner: Rc<dyn CommandRunner>,
    token: Rc<dyn TokenSource>,
}

impl GitTokenRunner {
    /// Runs through `inner`, with the token `token` answers.
    pub fn new(inner: Rc<dyn CommandRunner>, token: Rc<dyn TokenSource>) -> Self {
        Self { inner, token }
    }
}

impl CommandRunner for GitTokenRunner {
    // Without a token the command still runs: a public repository needs none, and a private one fails as git says.
    fn run(&self, command: &Command) -> CommandResult {
        let network = command.program() == "git"
            && command
                .args()
                .first()
                .is_some_and(|subcommand| NETWORK.contains(&subcommand.as_str()));
        let Some(token) = network.then(|| self.token.token().ok()).flatten() else {
            return self.inner.run(command);
        };
        let credential = Base64.standard(format!("x-access-token:{token}").as_bytes());
        let authorized = command
            .clone()
            .with_env("GIT_CONFIG_COUNT", "1")
            .with_env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
            .with_env(
                "GIT_CONFIG_VALUE_0",
                &format!("AUTHORIZATION: basic {credential}"),
            );
        self.inner.run(&authorized)
    }
}
