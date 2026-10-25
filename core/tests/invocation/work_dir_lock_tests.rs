// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::invocation::work_dir_lock::WorkDirLock;

#[test]
fn acquire_creates_a_missing_work_dir() {
    // Arrange
    let root = TempDir::new("lock-missing");
    let work_dir = root.path().join("not").join("yet");

    // Act
    let lock = WorkDirLock::acquire(&work_dir);

    // Assert
    assert!(lock.is_ok());
    assert!(work_dir.join("sentinel4rust.lock").is_file());
}

#[test]
fn acquire_twice_fails_until_the_first_is_dropped() {
    // Arrange
    let dir = TempDir::new("lock");

    // Act
    let first = WorkDirLock::acquire(dir.path()).unwrap();
    let error = WorkDirLock::acquire(dir.path()).err().unwrap();

    // Assert
    assert!(
        error.to_string().contains("another sentinel4rust"),
        "{error}"
    );

    // Act
    drop(first);

    // Assert
    assert!(WorkDirLock::acquire(dir.path()).is_ok());
}
