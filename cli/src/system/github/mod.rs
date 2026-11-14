// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The GitHub App, over `openssl` and `curl` (`ADR-GithubAppDiscovery`): its installation tokens, the API they
//! open, and discovery through it. Every secret is written to a private file under the App key's directory, which
//! the run unit cannot read, and never put on a command line.

pub mod app_token;
pub mod curl;
pub mod git_token_runner;
pub mod github_api;
pub mod github_app_discovery;
pub mod secret_dir;
pub mod token_source;
