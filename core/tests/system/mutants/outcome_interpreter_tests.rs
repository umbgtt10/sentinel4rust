// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_mutants_out::InMemoryMutantsOut;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::system::mutants::log_tail::LogTail;
use sentinel4rust::system::mutants::outcome_interpreter::OutcomeInterpreter;

fn exited(code: i32, stdout: &str) -> CommandResult {
    CommandResult::Exited {
        code: Some(code),
        stdout: stdout.to_string(),
        stderr: String::new(),
    }
}

fn interpreter() -> OutcomeInterpreter {
    OutcomeInterpreter::new(LogTail::new(200, 16 * 1024))
}

#[test]
fn interpret_each_exit_code_follows_the_table() {
    // Arrange
    let none = &InMemoryMutantsOut::default();
    let killed = CommandResult::Exited {
        code: None,
        stdout: String::new(),
        stderr: String::new(),
    };
    let timed_out = CommandResult::TimedOut {
        stdout: "a".to_string(),
        stderr: "b".to_string(),
    };

    // Act & Assert
    assert!(matches!(
        interpreter().interpret(&exited(4, "build failed"), none),
        RunOutcome::BaselineFailed { ref log_tail } if log_tail == "build failed"
    ));
    for code in [1, 5, 6, 70] {
        assert!(matches!(
            interpreter().interpret(&exited(code, ""), none),
            RunOutcome::ToolFailed { ref reason, .. } if reason == &format!("cargo-mutants exited with {code}")
        ));
    }
    assert!(
        matches!(interpreter().interpret(&killed, none), RunOutcome::ToolFailed { ref reason, .. } if reason.contains("signal"))
    );
    assert_eq!(
        interpreter().interpret(&timed_out, none),
        RunOutcome::ToolFailed {
            reason: "run cap hit".to_string(),
            log_tail: "a\nb".to_string()
        }
    );
    assert!(matches!(
        interpreter().interpret(&CommandResult::FailedToStart("no cargo".to_string()), none),
        RunOutcome::ToolFailed { ref reason, .. } if reason.contains("no cargo")
    ));
}
