// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! New survivors, checked once more before they are published.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::model::mutant_key::MutantKey;
use crate::model::run_outcome::RunOutcome;

/// Picks the survivors a run found that the previous record did not, and folds a second look at them back into the
/// run: a flaky test that let a mutant through once rarely does twice.
#[derive(Debug, Clone, Copy, Default)]
pub struct SurvivorRecheck;

impl SurvivorRecheck {
    /// `first` with every one of `rechecked` the re-check did not find missed or timed out counted as caught; when
    /// the re-check did not complete, `first` as it was. The test timeout stays `first`'s: the re-check is a run of
    /// its own, with a baseline and a timeout of its own.
    pub fn apply(
        &self,
        first: RunOutcome,
        rechecked: &[String],
        recheck: &RunOutcome,
    ) -> RunOutcome {
        let RunOutcome::Completed {
            missed: still_missed,
            timed_out: still_timed_out,
            ..
        } = recheck
        else {
            return first;
        };
        let RunOutcome::Completed {
            mut counts,
            missed,
            timed_out,
            test_timeout_seconds,
        } = first
        else {
            return first;
        };
        let caught: BTreeSet<&String> = rechecked
            .iter()
            .filter(|name| !still_missed.contains(name) && !still_timed_out.contains(name))
            .collect();
        let missed: Vec<String> = missed
            .into_iter()
            .filter(|name| !caught.contains(name))
            .collect();
        let newly_caught = u64::try_from(caught.len()).unwrap_or(u64::MAX);
        counts.missed = counts.missed.saturating_sub(newly_caught);
        counts.caught = counts.caught.saturating_add(newly_caught);
        RunOutcome::Completed {
            counts,
            missed,
            timed_out,
            test_timeout_seconds,
        }
    }

    /// The located names of the survivors of `outcome` whose keys are not in `previous_missed`; none unless it
    /// completed.
    pub fn new_survivors(
        &self,
        outcome: &RunOutcome,
        previous_missed: &[MutantKey],
    ) -> Vec<String> {
        let RunOutcome::Completed { missed, .. } = outcome else {
            return Vec::new();
        };
        let previous: BTreeSet<&MutantKey> = previous_missed.iter().collect();
        missed
            .iter()
            .zip(MutantKey::of_each(missed))
            .filter(|(_, key)| !previous.contains(key))
            .map(|(name, _)| name.clone())
            .collect()
    }
}
