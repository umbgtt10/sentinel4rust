// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What `git ls-remote` listed.

/// The refs `git ls-remote` listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteListing {
    refs: Vec<String>,
}

impl RemoteListing {
    /// Reads `hash<TAB>ref` lines; any other line is ignored.
    pub fn parse(text: &str) -> Self {
        Self {
            refs: text
                .lines()
                .filter_map(|line| {
                    line.split_once('\t')
                        .map(|(_, name)| name.trim().to_string())
                })
                .collect(),
        }
    }

    /// Whether `name` was listed.
    pub fn has(&self, name: &str) -> bool {
        self.refs.iter().any(|listed| listed == name)
    }
}
