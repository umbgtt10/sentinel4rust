// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What may name a watched repository and the branch tested in it.

/// The names a repository and its tested branch may have.
#[derive(Debug, Clone, Copy, Default)]
pub struct RepoNames;

impl RepoNames {
    /// A branch git would accept and that is not `mutants`, the earlier design's results branch.
    pub fn is_branch(&self, branch: &str) -> bool {
        !branch.is_empty()
            && branch != "mutants"
            && !branch.starts_with('-')
            && !branch.contains("..")
            && branch
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
    }

    /// A name that can name a clone: letters, digits, `-`, `_` and `.`, not starting with `.`.
    pub fn is_name(&self, name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('.')
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }
}
