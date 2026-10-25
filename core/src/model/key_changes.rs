// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! How a list of survivors changed.

/// How many keys appeared and disappeared between two lists.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyChanges {
    /// Keys in the current list but not the previous one.
    pub new: u64,
    /// Keys in the previous list but not the current one.
    pub gone: u64,
}
