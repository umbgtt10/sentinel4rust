// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::system::mutants::mutants_out_lists::MutantsOutLists;
use std::fs;

fn lists(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new("mutants-out");
    for (name, text) in files {
        fs::write(dir.path().join(name), text).unwrap();
    }
    dir
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
