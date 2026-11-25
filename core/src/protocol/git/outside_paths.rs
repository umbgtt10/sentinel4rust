// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Path dependencies that leave the repository.

use crate::model::toml::comment_stripper::CommentStripper;
use alloc::vec::Vec;

/// Finds `path = "..."` values in a manifest that point outside the repository: a run's checkout holds only the
/// repository, so such a dependency is never there.
#[derive(Debug, Clone, Copy, Default)]
pub struct OutsidePaths;

impl OutsidePaths {
    /// Whether `text`, the manifest at `manifest` (a path inside the repository), names a `path` that leaves it:
    /// absolute, or with more `..` than the manifest is deep.
    pub fn escapes(&self, manifest: &str, text: &str) -> bool {
        let depth = manifest.matches('/').count();
        text.lines()
            .map(|raw| CommentStripper.code(raw).unwrap_or(raw))
            .flat_map(Self::path_values)
            .any(|value| Self::leaves(depth, value))
    }

    /// Whether a `path` after `before` is a whole key, not the end of a longer one.
    fn is_key(before: &str) -> bool {
        before
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '"')))
    }

    fn leaves(depth: usize, value: &str) -> bool {
        if value.starts_with('/') {
            return true;
        }
        let mut depth = depth;
        for part in value.split('/') {
            match part {
                "" | "." => {}
                ".." if depth == 0 => return true,
                ".." => depth -= 1,
                _ => depth += 1,
            }
        }
        false
    }

    fn path_values(line: &str) -> Vec<&str> {
        line.match_indices("path")
            .filter(|(at, _)| Self::is_key(&line[..*at]))
            .filter_map(|(at, key)| Self::quoted_value(&line[at + key.len()..]))
            .collect()
    }

    /// The quoted value of the `= "value"` that `after` starts with.
    fn quoted_value(after: &str) -> Option<&str> {
        let value = after.trim_start().strip_prefix('=')?.trim_start();
        let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;
        let end = value[1..].find(quote)?;
        Some(&value[1..=end])
    }
}
