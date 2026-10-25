// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::collections::BTreeMap;
use xtask::files::file_failure::FileFailure;
use xtask::files::file_store::FileStore;

// The seam the stern gate reads and writes its install marker through. Lives
// here, beside the trait it implements, as the command runner's fake does.
pub struct FakeFileStore {
    files: BTreeMap<String, String>,
    unwritable: bool,
    written: RefCell<BTreeMap<String, String>>,
}

impl FakeFileStore {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            unwritable: false,
            written: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files
            .insert(String::from(path), String::from(contents));
        self
    }

    pub fn with_unwritable(mut self, unwritable: bool) -> Self {
        self.unwritable = unwritable;
        self
    }

    pub fn written(&self) -> BTreeMap<String, String> {
        self.written.borrow().clone()
    }
}

impl Default for FakeFileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FileStore for FakeFileStore {
    fn is_file(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    fn read(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn write(&self, path: &str, contents: &str) -> Result<(), FileFailure> {
        if self.unwritable {
            return Err(FileFailure {
                path: String::from(path),
                reason: String::from("denied"),
            });
        }
        self.written
            .borrow_mut()
            .insert(String::from(path), String::from(contents));
        Ok(())
    }
}

#[test]
fn is_file_of_a_path_it_was_given_returns_true() {
    // Arrange
    let files = FakeFileStore::new().with_file("bin/tool", "");

    // Act
    let present = files.is_file("bin/tool");

    // Assert
    assert!(present);
}

#[test]
fn read_of_a_path_it_was_not_given_returns_none() {
    // Arrange
    let files = FakeFileStore::new().with_file("REV", "abc\n");

    // Act
    let contents = files.read("other");

    // Assert
    assert_eq!(contents, None);
}

#[test]
fn write_through_a_trait_object_records_the_contents_by_path() {
    // Arrange
    let files = FakeFileStore::new();
    let as_trait_object: &dyn FileStore = &files;

    // Act
    let result = as_trait_object.write("REV", "abc\n");

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        files.written(),
        BTreeMap::from([(String::from("REV"), String::from("abc\n"))])
    );
}

#[test]
fn write_when_unwritable_returns_a_failure_naming_the_path() {
    // Arrange
    let files = FakeFileStore::new().with_unwritable(true);

    // Act
    let result = files.write("REV", "abc\n");

    // Assert
    assert_eq!(
        result,
        Err(FileFailure {
            path: String::from("REV"),
            reason: String::from("denied"),
        })
    );
    assert!(files.written().is_empty());
}
