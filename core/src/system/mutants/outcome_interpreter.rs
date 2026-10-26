// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What a cargo-mutants run ended as.

use crate::model::command_result::CommandResult;
use crate::model::run_outcome::RunOutcome;
use crate::system::mutants::log_tail::LogTail;
use crate::system::mutants::mutants_out_lists::MutantsOutLists;
use crate::traits::mutants_out::MutantsOut;

/// Turns the process result and `mutants.out` into an outcome (the exit-code table of `ADR-ResultsFailuresAndBackOff`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutcomeInterpreter {
    tail: LogTail,
}

impl OutcomeInterpreter {
    /// Failures keep `tail` of the run's output.
    pub fn new(tail: LogTail) -> Self {
        Self { tail }
    }

    /// 0, 2 and 3 completed (the lists say what), 4 is a failing baseline, anything else the tool failing.
    pub fn interpret(&self, result: &CommandResult, mutants_out: &dyn MutantsOut) -> RunOutcome {
        match result {
            CommandResult::FailedToStart(message) => RunOutcome::ToolFailed {
                reason: format!("could not start cargo-mutants: {message}"),
                log_tail: String::new(),
            },
            CommandResult::TimedOut { stdout, stderr } => RunOutcome::ToolFailed {
                reason: "run cap hit".to_string(),
                log_tail: self.tail.of(stdout, stderr),
            },
            CommandResult::Exited {
                code,
                stdout,
                stderr,
            } => match code {
                Some(0 | 2 | 3) => MutantsOutLists.read(mutants_out).unwrap_or_else(|error| {
                    RunOutcome::ToolFailed {
                        reason: error.to_string(),
                        log_tail: self.tail.of(stdout, stderr),
                    }
                }),
                Some(4) => RunOutcome::BaselineFailed {
                    log_tail: self.tail.of(stdout, stderr),
                },
                Some(code) => RunOutcome::ToolFailed {
                    reason: format!("cargo-mutants exited with {code}"),
                    log_tail: self.tail.of(stdout, stderr),
                },
                None => RunOutcome::ToolFailed {
                    reason: "cargo-mutants was killed by a signal".to_string(),
                    log_tail: self.tail.of(stdout, stderr),
                },
            },
        }
    }
}
