// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One onboarded repository.

use alloc::string::String;
use core::time::Duration;

/// One onboarded repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardedRepo {
    /// Unique; names the local clone.
    pub name: String,
    /// What git fetches from and pushes to.
    pub url: String,
    /// The branch tested; `main` by default.
    pub branch: String,
    /// cargo-mutants `--jobs` for this repository.
    pub jobs: u32,
    /// Cap on one cargo-mutants run of this repository.
    pub run_timeout: Duration,
}
