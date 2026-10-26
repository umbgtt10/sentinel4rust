// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::system::mutants::unreadable_list::UnreadableList;
use sentinel4rust::traits::mutants_out::MutantsOut;
use std::collections::BTreeMap;

// The lists of one run, held in memory; a list it does not hold cannot be
// read, as a file that is not there.
#[derive(Default)]
pub struct InMemoryMutantsOut {
    lists: BTreeMap<String, String>,
}

impl InMemoryMutantsOut {
    pub fn with_lists(files: &[(&str, &str)]) -> Self {
        Self {
            lists: files
                .iter()
                .map(|(name, text)| ((*name).to_string(), (*text).to_string()))
                .collect(),
        }
    }
}

impl MutantsOut for InMemoryMutantsOut {
    fn list(&self, name: &str) -> Result<String, UnreadableList> {
        self.lists.get(name).cloned().ok_or_else(|| UnreadableList {
            name: name.to_string(),
            reason: String::from("absent"),
        })
    }
}
