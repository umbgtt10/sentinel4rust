// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The GitHub App's installation token: a JWT signed with `openssl`, exchanged with `curl`, kept until it nears
//! its expiry.

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::time::Duration;

use crate::system::github::curl::Curl;
use crate::system::github::secret_dir::SecretDir;
use crate::system::github::token_source::TokenSource;
use sentinel4rust::model::command::Command;
use sentinel4rust::model::github::app_jwt::AppJwt;
use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust::model::onboarding::github_app::GithubApp;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::traits::clock::Clock;
use sentinel4rust::traits::command_runner::CommandRunner;
use serde_json::Value;
use serde_json::from_str;

/// A token is minted again this long before it expires.
const RENEW_BEFORE: Duration = Duration::from_secs(300);
/// The deadline for signing.
const SIGN_TIMEOUT: Duration = Duration::from_secs(30);

/// The installation token, minted when there is none or it nears its expiry.
pub struct AppToken {
    runner: Rc<dyn CommandRunner>,
    clock: Rc<dyn Clock>,
    app: GithubApp,
    secrets: SecretDir,
    curl: Curl,
    cached: RefCell<Option<(String, Timestamp)>>,
}

impl AppToken {
    /// For `app`, its files under `secrets`, signing and asking through `runner`.
    pub fn new(
        runner: Rc<dyn CommandRunner>,
        clock: Rc<dyn Clock>,
        app: GithubApp,
        secrets: SecretDir,
    ) -> Self {
        Self {
            curl: Curl::new(runner.clone()),
            runner,
            clock,
            app,
            secrets,
            cached: RefCell::new(None),
        }
    }

    /// The App's JWT for now, signed with its private key by `openssl`.
    pub fn jwt(&self) -> Result<String, DiscoveryError> {
        let jwt = AppJwt::new(self.app.app_id);
        let input = jwt.signing_input(self.clock.now());
        let input_path = self.secrets.write("jwt.input", input.as_bytes())?;
        let signature_path = self.secrets.path("jwt.sig");
        let (input_text, signature_text) = (
            input_path.to_string_lossy().into_owned(),
            signature_path.to_string_lossy().into_owned(),
        );
        let args = [
            "dgst",
            "-sha256",
            "-sign",
            self.app.key.as_str(),
            "-out",
            signature_text.as_str(),
            input_text.as_str(),
        ];
        let dir = input_path
            .parent()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        let result = self
            .runner
            .run(&Command::new("openssl", &args, &dir, SIGN_TIMEOUT));
        if result.success().is_none() {
            return Err(DiscoveryError(format!("openssl: {}", result.describe())));
        }
        let signature = fs::read(&signature_path)
            .map_err(|e| DiscoveryError(format!("read {}: {e}", signature_path.display())))?;
        Ok(jwt.token(&input, &signature))
    }

    fn mint(&self) -> Result<(String, Timestamp), DiscoveryError> {
        let jwt = self.jwt()?;
        let header = self.secrets.write(
            "app.header",
            format!("Authorization: Bearer {jwt}\n").as_bytes(),
        )?;
        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            self.app.installation_id
        );
        let body = self.curl.post(&header, &url)?;
        let answer: Value = from_str(&body).map_err(|e| DiscoveryError(format!("{url}: {e}")))?;
        let token = answer["token"].as_str();
        let expires = answer["expires_at"].as_str().and_then(Timestamp::parse);
        match (token, expires) {
            (Some(token), Some(expires)) => Ok((token.to_string(), expires)),
            _ => Err(DiscoveryError(format!(
                "{url}: no token and expires_at in the answer"
            ))),
        }
    }
}

impl TokenSource for AppToken {
    fn token(&self) -> Result<String, DiscoveryError> {
        let cached = self.cached.borrow().clone();
        if let Some((token, expires)) = cached
            && self.clock.now().plus(RENEW_BEFORE) < expires
        {
            return Ok(token);
        }
        let (token, expires) = self.mint()?;
        *self.cached.borrow_mut() = Some((token.clone(), expires));
        Ok(token)
    }
}
