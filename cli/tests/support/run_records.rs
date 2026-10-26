// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::branch::run_record::RunRecord;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::timestamp::Timestamp;

// A record for commit abab..., config_version 2, with every field set.
pub fn record(outcome: OutcomeKind) -> RunRecord {
    RunRecord {
        commit: CommitHash::parse(&"ab".repeat(20)).unwrap(),
        config_version: 2,
        kind: KIND_FULL.to_string(),
        outcome,
        started: Timestamp::from_unix(1_792_800_000),
        finished: Timestamp::from_unix(1_792_803_600),
        counts: MutantCounts {
            caught: 10,
            missed: 2,
            timeout: 1,
            unviable: 3,
        },
        new: 1,
        gone: 0,
        cargo_mutants_version: "cargo-mutants 27.1.0".to_string(),
        rustc_version: "rustc 1.97.0".to_string(),
        arguments: "mutants --jobs 2".to_string(),
        reason: None,
        log_tail: None,
    }
}
