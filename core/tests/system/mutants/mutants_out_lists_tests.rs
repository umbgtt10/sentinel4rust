// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::system::mutants::mutants_out_lists::MutantsOutLists;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

// The four lists cargo-mutants 27.1.0 wrote on 2026-10-25, unedited, for the
// two fixture crates the stage-2 tests in main_tests.rs run: one whose test
// fails without any mutation (exit 4), and `spin`, a loop behind `black_box`
// whose `+ with *` mutant never ends (exit 3).
fn captured(case: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/mutants_out")
        .join(case)
}

fn lists(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new("mutants-out");
    for (name, text) in files {
        fs::write(dir.path().join(name), text).unwrap();
    }
    dir
}

// A failing baseline still writes every list, empty; exit 4 is read as
// baseline-failed before the lists are, so they never count.
#[test]
fn read_of_a_real_baseline_failure_finds_every_list_empty() {
    // Arrange
    let dir = captured("baseline_failed");

    // Act
    let outcome = MutantsOutLists.read(&dir);

    // Assert
    assert_eq!(
        outcome,
        Ok(RunOutcome::Completed {
            counts: MutantCounts {
                caught: 0,
                missed: 0,
                timeout: 0,
                unviable: 0,
            },
            missed: vec![],
            timed_out: vec![],
        })
    );
}

#[test]
fn read_of_a_real_run_with_a_timeout_keeps_its_line_and_counts_the_rest() {
    // Arrange
    let dir = captured("timeouts");

    // Act
    let outcome = MutantsOutLists.read(&dir);

    // Assert
    assert_eq!(
        outcome,
        Ok(RunOutcome::Completed {
            counts: MutantCounts {
                caught: 6,
                missed: 0,
                timeout: 1,
                unviable: 0,
            },
            missed: vec![],
            timed_out: vec!["src/lib.rs:6:25: replace + with * in spin".to_string()],
        })
    );
}

#[test]
fn read_of_all_four_lists_counts_each_and_keeps_missed_and_timeouts() {
    // Arrange
    let dir = lists(&[
        ("caught.txt", "src/a.rs:1:1: c1\nsrc/a.rs:2:1: c2\n"),
        ("missed.txt", "src/a.rs:3:1: m1  \n\n"),
        ("timeout.txt", "src/a.rs:5:1: t1\n"),
        (
            "unviable.txt",
            "src/a.rs:4:1: u1\nsrc/a.rs:6:1: u2\nsrc/a.rs:7:1: u3\n",
        ),
    ]);

    // Act
    let outcome = MutantsOutLists.read(dir.path());

    // Assert
    assert_eq!(
        outcome,
        Ok(RunOutcome::Completed {
            counts: MutantCounts {
                caught: 2,
                missed: 1,
                timeout: 1,
                unviable: 3,
            },
            missed: vec!["src/a.rs:3:1: m1".to_string()],
            timed_out: vec!["src/a.rs:5:1: t1".to_string()],
        })
    );
}

#[test]
fn read_without_missed_names_the_list() {
    // Arrange
    let dir = lists(&[("caught.txt", "")]);

    // Act
    let outcome = MutantsOutLists.read(dir.path());

    // Assert
    assert!(outcome.is_err_and(|error| error.name == "missed.txt"));
}

#[test]
fn read_without_the_optional_lists_counts_them_as_none() {
    // Arrange
    let dir = lists(&[("caught.txt", ""), ("missed.txt", "")]);

    // Act
    let outcome = MutantsOutLists.read(dir.path());

    // Assert
    assert_eq!(
        outcome,
        Ok(RunOutcome::Completed {
            counts: MutantCounts {
                caught: 0,
                missed: 0,
                timeout: 0,
                unviable: 0,
            },
            missed: vec![],
            timed_out: vec![],
        })
    );
}
