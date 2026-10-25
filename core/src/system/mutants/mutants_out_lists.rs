// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The lists cargo-mutants writes into `mutants.out`.

use std::fs;
use std::path::Path;

use crate::model::counts::MutantCounts;
use crate::model::run_outcome::RunOutcome;
use crate::system::mutants::unreadable_list::UnreadableList;

/// Reads `caught.txt` and `missed.txt` (required) and `timeout.txt` and `unviable.txt` (absent when empty).
#[derive(Debug, Clone, Copy, Default)]
pub struct MutantsOutLists;

impl MutantsOutLists {
    /// A completed outcome with the counts and the missed and timed-out mutants, as written.
    pub fn read(&self, dir: &Path) -> Result<RunOutcome, UnreadableList> {
        let caught = Self::list(dir, "caught.txt", true)?;
        let missed = Self::list(dir, "missed.txt", true)?;
        let timed_out = Self::list(dir, "timeout.txt", false)?;
        let unviable = Self::list(dir, "unviable.txt", false)?;
        let count = |list: &[String]| u64::try_from(list.len()).unwrap_or(u64::MAX);
        Ok(RunOutcome::Completed {
            counts: MutantCounts {
                caught: count(&caught),
                missed: count(&missed),
                timeout: count(&timed_out),
                unviable: count(&unviable),
            },
            missed,
            timed_out,
        })
    }

    fn list(dir: &Path, name: &str, required: bool) -> Result<Vec<String>, UnreadableList> {
        match fs::read_to_string(dir.join(name)) {
            Ok(text) => Ok(text
                .lines()
                .map(str::trim_end)
                .filter(|l| !l.is_empty())
                .map(ToString::to_string)
                .collect()),
            Err(_) if !required => Ok(Vec::new()),
            Err(error) => Err(UnreadableList {
                name: name.to_string(),
                reason: error.to_string(),
            }),
        }
    }
}
