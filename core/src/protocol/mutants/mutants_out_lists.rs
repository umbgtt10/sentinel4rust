// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The lists cargo-mutants writes into `mutants.out`.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::model::counts::MutantCounts;
use crate::model::run_outcome::RunOutcome;
use crate::protocol::mutants::test_timeout_line::TestTimeoutLine;
use crate::protocol::mutants::unreadable_list::UnreadableList;
use crate::traits::mutants_out::MutantsOut;

/// Reads `caught.txt` and `missed.txt` (required) and `timeout.txt` and `unviable.txt` (absent when empty), and the
/// test timeout from `debug.log` (none when it is absent or does not say it).
#[derive(Debug, Clone, Copy, Default)]
pub struct MutantsOutLists;

impl MutantsOutLists {
    /// A completed outcome with the counts, the missed and timed-out mutants, as written, and the test timeout.
    pub fn read(&self, lists: &dyn MutantsOut) -> Result<RunOutcome, UnreadableList> {
        let caught = Self::list(lists, "caught.txt", true)?;
        let missed = Self::list(lists, "missed.txt", true)?;
        let timed_out = Self::list(lists, "timeout.txt", false)?;
        let unviable = Self::list(lists, "unviable.txt", false)?;
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
            test_timeout_seconds: lists
                .list("debug.log")
                .ok()
                .and_then(|log| TestTimeoutLine.seconds(&log)),
        })
    }

    fn list(
        lists: &dyn MutantsOut,
        name: &str,
        required: bool,
    ) -> Result<Vec<String>, UnreadableList> {
        match lists.list(name) {
            Ok(text) => Ok(text
                .lines()
                .map(str::trim_end)
                .filter(|l| !l.is_empty())
                .map(ToString::to_string)
                .collect()),
            Err(_) if !required => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }
}
