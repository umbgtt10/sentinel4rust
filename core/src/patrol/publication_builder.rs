// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What a run publishes.

use crate::model::branch::branch_constants::KIND_FULL;
use crate::model::branch::publication::Publication;
use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;
use crate::model::counts::MutantCounts;
use crate::model::mutant_key::MutantKey;
use crate::model::mutation_run::MutationRun;
use crate::model::outcome_kind::OutcomeKind;
use crate::model::run_outcome::RunOutcome;
use crate::model::timestamp::Timestamp;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Builds what to publish for a run that is to be published, under the roster's `config_version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicationBuilder {
    config_version: i64,
}

impl PublicationBuilder {
    /// Records carry `config_version`.
    pub fn new(config_version: i64) -> Self {
        Self { config_version }
    }

    /// The publication for `run` of `head`, between `started` and `finished`; new and gone are counted
    /// against `previous_missed`, and only a completed run publishes lists.
    pub fn build(
        &self,
        run: &MutationRun,
        head: &CommitHash,
        started: Timestamp,
        finished: Timestamp,
        previous_missed: &[MutantKey],
    ) -> Publication {
        let record =
            |outcome, counts, (new, gone), test_timeout_seconds, (reason, log_tail)| RunRecord {
                commit: head.clone(),
                config_version: self.config_version,
                kind: KIND_FULL.to_string(),
                outcome,
                started,
                finished,
                counts,
                new,
                gone,
                cargo_mutants_version: run.cargo_mutants_version.clone(),
                rustc_version: run.rustc_version.clone(),
                arguments: run.arguments.clone(),
                test_timeout_seconds,
                reason,
                log_tail,
            };
        match &run.outcome {
            RunOutcome::Completed {
                counts,
                missed,
                timed_out,
                test_timeout_seconds,
            } => {
                let missed_keys = MutantKey::from_located(missed);
                let changes = MutantKey::changes(previous_missed, &missed_keys);
                let new_gone = (changes.new, changes.gone);
                Publication {
                    record: record(
                        OutcomeKind::Completed,
                        *counts,
                        new_gone,
                        *test_timeout_seconds,
                        (None, None),
                    ),
                    missed: missed_keys,
                    timed_out: MutantKey::from_located(timed_out),
                    located_missed: Self::sorted(missed),
                    located_timed_out: Self::sorted(timed_out),
                }
            }
            RunOutcome::BaselineFailed { log_tail } => Self::without_lists(record(
                OutcomeKind::BaselineFailed,
                MutantCounts::default(),
                (0, 0),
                None,
                (None, Some(log_tail.clone())),
            )),
            RunOutcome::ToolchainMissing { message } => Self::without_lists(record(
                OutcomeKind::ToolFailed,
                MutantCounts::default(),
                (0, 0),
                None,
                (Some(message.clone()), None),
            )),
            RunOutcome::ToolFailed { reason, log_tail } => Self::without_lists(record(
                OutcomeKind::ToolFailed,
                MutantCounts::default(),
                (0, 0),
                None,
                (Some(reason.clone()), Some(log_tail.clone())),
            )),
        }
    }

    fn sorted(lines: &[String]) -> Vec<String> {
        let mut lines = lines.to_vec();
        lines.sort();
        lines
    }

    fn without_lists(record: RunRecord) -> Publication {
        Publication {
            record,
            missed: Vec::new(),
            timed_out: Vec::new(),
            located_missed: Vec::new(),
            located_timed_out: Vec::new(),
        }
    }
}
