// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::run_records::record;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::summary_toml::SummaryToml;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::system::git::git_runner::GitRunner;
use sentinel4rust::system::git::record_reader::RecordReader;
use std::path::Path;
use std::rc::Rc;

#[test]
fn previous_missed_reads_one_key_per_non_blank_line() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok(
        "src/lib.rs: a\n\n  \nsrc/lib.rs: b\n",
    )]));
    let reader = RecordReader::new(GitRunner::new(runner.clone()));

    // Act
    let missed = reader.previous_missed(Path::new("/w/repos/x.git"));

    // Assert
    assert_eq!(
        missed,
        [
            MutantKey::new("src/lib.rs: a"),
            MutantKey::new("src/lib.rs: b")
        ]
    );
    assert_eq!(
        runner.args(),
        [["show", "refs/remotes/origin/mutants:missed.txt"]]
    );
}

#[test]
fn previous_missed_without_a_list_is_empty() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![failed(128, "fatal: path")]));
    let reader = RecordReader::new(GitRunner::new(runner));

    // Act
    let missed = reader.previous_missed(Path::new("/w/repos/x.git"));

    // Assert
    assert_eq!(missed, Vec::<MutantKey>::new());
}

#[test]
fn record_of_a_valid_summary_parses_it() {
    // Arrange
    let written = record(OutcomeKind::Completed);
    let runner = Rc::new(ScriptedRunner::new(vec![ok(&SummaryToml.render(&written))]));
    let reader = RecordReader::new(GitRunner::new(runner.clone()));

    // Act
    let read = reader.record(Path::new("/w/repos/x.git"));

    // Assert
    assert_eq!(read, RecordState::Present(Box::new(written)));
    assert_eq!(
        runner.args(),
        [["show", "refs/remotes/origin/mutants:summary.toml"]]
    );
}

// Written by a newer version of the tool: run the commit again, as before.
#[test]
fn record_of_an_unknown_schema_is_absent() {
    // Arrange
    let text = SummaryToml
        .render(&record(OutcomeKind::Completed))
        .replace("schema = 1", "schema = 2");
    let runner = Rc::new(ScriptedRunner::new(vec![ok(&text)]));
    let reader = RecordReader::new(GitRunner::new(runner));

    // Act
    let read = reader.record(Path::new("/w/repos/x.git"));

    // Assert
    assert_eq!(read, RecordState::Absent);
}

#[test]
fn record_of_an_unparseable_summary_is_unreadable() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok("schema = [")]));
    let reader = RecordReader::new(GitRunner::new(runner));

    // Act
    let read = reader.record(Path::new("/w/repos/x.git"));

    // Assert
    assert!(matches!(
        read,
        RecordState::Unreadable(RecordError::Invalid(_))
    ));
}

#[test]
fn record_without_a_summary_is_absent() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![failed(128, "fatal: path")]));
    let reader = RecordReader::new(GitRunner::new(runner));

    // Act
    let read = reader.record(Path::new("/w/repos/x.git"));

    // Assert
    assert_eq!(read, RecordState::Absent);
}
