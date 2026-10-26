// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Position-free mutant identities (`ADR-PositionFreeMutantKeys`).
//!
//! cargo-mutants names a mutant `src/lib.rs:12:5: replace == with != in Parser::next`. The position changes
//! whenever a line is added above it, so the published lists drop it: the key is
//! `src/lib.rs: replace == with != in Parser::next`, with ` #2`, ` #3`… when the same key occurs more than
//! once in a file, numbered in source order.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::model::key_changes::KeyChanges;
use crate::model::located::Located;

/// A mutant's identity without line and column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MutantKey(String);

impl MutantKey {
    /// Keys for a list of located mutant names, as cargo-mutants writes them; returned sorted.
    pub fn from_located(lines: &[String]) -> Vec<Self> {
        let mut keys = Self::of_each(lines);
        keys.sort();
        keys
    }

    /// A key from its published text.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The key of each located mutant name, in the order given; ordinals count in source order.
    pub fn of_each(lines: &[String]) -> Vec<Self> {
        let mut order: Vec<(Located, usize)> = lines
            .iter()
            .enumerate()
            .map(|(index, line)| (Located::parse(line), index))
            .collect();
        order.sort();
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        let mut keys = vec![Self(String::new()); lines.len()];
        for (located, index) in order {
            let base = located.base();
            let count = seen.entry(base.clone()).or_insert(0);
            *count += 1;
            keys[index] = if *count == 1 {
                Self(base)
            } else {
                Self(format!("{base} #{count}"))
            };
        }
        keys
    }

    /// The key's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// How many keys appeared and disappeared between two lists.
    pub fn changes(previous: &[Self], current: &[Self]) -> KeyChanges {
        let previous: BTreeSet<&Self> = previous.iter().collect();
        let current: BTreeSet<&Self> = current.iter().collect();
        let count = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
        KeyChanges {
            new: count(current.difference(&previous).count()),
            gone: count(previous.difference(&current).count()),
        }
    }
}

impl fmt::Display for MutantKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
