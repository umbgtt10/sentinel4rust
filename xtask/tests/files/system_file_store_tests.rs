// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::temp_dir;
use std::fs::remove_file;
use std::process::id;
use xtask::files::file_store::FileStore;
use xtask::files::system_file_store::SystemFileStore;

fn scratch_path(case: &str) -> String {
    temp_dir()
        .join(format!("xtask-system-file-store-{}-{case}", id()))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn is_file_of_a_directory_returns_false() {
    // Arrange
    let files = SystemFileStore::new();
    let directory = temp_dir().to_string_lossy().into_owned();

    // Act
    let present = files.is_file(&directory);

    // Assert
    assert!(!present);
}

#[test]
fn is_file_of_a_written_file_returns_true() {
    // Arrange
    let files = SystemFileStore::new();
    let path = scratch_path("is-file");
    files.write(&path, "").expect("the temp dir is writable");

    // Act
    let present = files.is_file(&path);

    // Assert
    assert!(present);
    let _ = remove_file(&path);
}

#[test]
fn read_of_a_file_that_does_not_exist_returns_none() {
    // Arrange
    let files = SystemFileStore::new();

    // Act
    let contents = files.read(&scratch_path("never-written"));

    // Assert
    assert_eq!(contents, None);
}

#[test]
fn write_into_a_directory_that_does_not_exist_returns_a_failure_naming_the_path() {
    // Arrange
    let files = SystemFileStore::new();
    let path = format!("{}/REV", scratch_path("no-such-directory"));

    // Act
    let result = files.write(&path, "8b0e8e5\n");

    // Assert
    assert!(result.is_err_and(|failure| failure.path == path));
}

#[test]
fn write_then_read_returns_the_contents() {
    // Arrange
    let files = SystemFileStore::new();
    let path = scratch_path("round-trip");

    // Act
    let result = files.write(&path, "8b0e8e5\n");

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(files.read(&path), Some(String::from("8b0e8e5\n")));
    let _ = remove_file(&path);
}
