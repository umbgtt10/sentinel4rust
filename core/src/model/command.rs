// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A program to run.

use std::collections::BTreeMap;
use std::iter::once;
use std::time::Duration;

/// A program to run: what, with which arguments, where, with which extra environment, for how long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    program: String,
    args: Vec<String>,
    dir: String,
    env: BTreeMap<String, String>,
    timeout: Duration,
    capture_limit: Option<usize>,
}

impl Command {
    /// A command with no extra environment.
    pub fn new(program: &str, args: &[&str], dir: &str, timeout: Duration) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(ToString::to_string).collect(),
            dir: dir.to_string(),
            env: BTreeMap::new(),
            timeout,
            capture_limit: None,
        }
    }

    /// Its arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// How many of the last bytes of each output stream are kept; `None`: all of it.
    pub fn capture_limit(&self) -> Option<usize> {
        self.capture_limit
    }

    /// The working directory.
    pub fn dir(&self) -> &str {
        &self.dir
    }

    /// The command line, for logs.
    pub fn display(&self) -> String {
        once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extra environment variables.
    pub fn env(&self) -> &BTreeMap<String, String> {
        &self.env
    }

    /// The executable.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Killed, with its process group, after this long.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// The same command, keeping only the last `bytes` of each output stream.
    #[must_use]
    pub fn with_capture_limit(self, bytes: usize) -> Self {
        Self {
            capture_limit: Some(bytes),
            ..self
        }
    }

    /// The same command with one more environment variable.
    #[must_use]
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }
}
