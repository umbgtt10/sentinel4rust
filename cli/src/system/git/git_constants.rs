// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the git parts share.

use std::time::Duration;

/// Deadline for commands that talk to the remote, and for creating a checkout.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(5 * 60);
/// Deadline for other git commands.
pub const LOCAL_TIMEOUT: Duration = Duration::from_secs(60);
/// Where the fetched `mutants` branch is kept in the bare clone.
pub const MUTANTS_REMOTE_REF: &str = "refs/remotes/origin/mutants";
