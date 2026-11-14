// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! GitHub's API through `curl`, its authorization read from a file.

use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust::traits::command_runner::CommandRunner;

/// curl's own limit on one request, in seconds.
const MAX_TIME: &str = "60";
/// The deadline for one curl.
const TIMEOUT: Duration = Duration::from_secs(90);

/// One request to GitHub's API with curl, the `Authorization` header read from a private file.
#[derive(Clone)]
pub struct Curl {
    runner: Rc<dyn CommandRunner>,
}

impl Curl {
    /// curl, run by `runner`.
    pub fn new(runner: Rc<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    /// GETs `url`; its body, or how it failed, GitHub's answer included.
    pub fn get(&self, header: &Path, url: &str) -> Result<String, DiscoveryError> {
        self.body(&["--fail-with-body", "--max-time", MAX_TIME], header, url)
    }

    /// POSTs to `url`; its body, or how it failed, GitHub's answer included.
    pub fn post(&self, header: &Path, url: &str) -> Result<String, DiscoveryError> {
        self.body(
            &[
                "--fail-with-body",
                "--max-time",
                MAX_TIME,
                "--request",
                "POST",
            ],
            header,
            url,
        )
    }

    /// The HTTP status of a GET of `url`, as curl writes it.
    pub fn status(&self, header: &Path, url: &str) -> Result<String, DiscoveryError> {
        self.body(
            &[
                "--max-time",
                MAX_TIME,
                "--output",
                "/dev/null",
                "--write-out",
                "%{http_code}",
            ],
            header,
            url,
        )
    }

    fn body(&self, options: &[&str], header: &Path, url: &str) -> Result<String, DiscoveryError> {
        let header = format!("@{}", header.display());
        let mut args = vec!["--silent", "--show-error"];
        args.extend(options);
        args.extend([
            "--header",
            header.as_str(),
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            url,
        ]);
        let dir = header.trim_start_matches('@');
        let dir = Path::new(dir).parent().unwrap_or(Path::new("/"));
        let result = self.runner.run(&Command::new(
            "curl",
            &args,
            &dir.to_string_lossy(),
            TIMEOUT,
        ));
        match &result {
            CommandResult::Exited {
                code: Some(0),
                stdout,
                ..
            } => Ok(stdout.clone()),
            CommandResult::Exited { stdout, .. } if !stdout.trim().is_empty() => {
                Err(DiscoveryError(format!(
                    "curl {url}: {}: {}",
                    result.describe(),
                    stdout.trim()
                )))
            }
            _ => Err(DiscoveryError(format!("curl {url}: {}", result.describe()))),
        }
    }
}
