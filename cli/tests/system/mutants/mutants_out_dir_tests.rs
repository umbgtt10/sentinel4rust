// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::traits::mutants_out::MutantsOut;
use sentinel4rust_cli::system::mutants::mutants_out_dir::MutantsOutDir;
use std::fs;

#[test]
fn list_of_a_file_in_the_directory_returns_its_text() {
    // Arrange
    let dir = TempDir::new("mutants-out-dir");
    fs::write(dir.path().join("missed.txt"), "src/a.rs:3:1: m1\n").unwrap();
    let lists = MutantsOutDir::new(dir.path().to_path_buf());

    // Act
    let text = lists.list("missed.txt");

    // Assert
    assert_eq!(text, Ok(String::from("src/a.rs:3:1: m1\n")));
}

#[test]
fn list_of_a_file_that_is_not_there_names_it_and_why() {
    // Arrange
    let dir = TempDir::new("mutants-out-dir-empty");
    let lists = MutantsOutDir::new(dir.path().to_path_buf());

    // Act
    let error = lists.list("caught.txt").unwrap_err();

    // Assert
    assert_eq!(error.name, "caught.txt");
    assert!(error.reason.contains("No such file"), "{}", error.reason);
}
