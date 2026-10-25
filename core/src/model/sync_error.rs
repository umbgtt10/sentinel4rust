// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why a fetch failed.

/// Why a fetch failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// Repository not found or permission denied.
    Unreachable(String),
    /// Anything else, likely temporary.
    Transient(String),
    /// The configured branch does not exist on the remote.
    NoBranch,
}
