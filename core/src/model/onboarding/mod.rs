// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The roster: which repositories to watch, and how (`deploy/roster.example.toml`).

pub mod github_app;
pub mod github_app_toml;
pub mod onboarded_repo;
pub mod repo_names;
pub mod roster;
pub mod roster_error;
pub mod toml_roster;
