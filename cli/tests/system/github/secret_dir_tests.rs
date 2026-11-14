// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust_cli::system::github::secret_dir::SecretDir;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[test]
fn write_creates_a_private_file_in_a_private_directory() {
    // Arrange
    let root = TempDir::new("secrets");
    let secrets = SecretDir::new(root.path().join("github"));

    // Act
    let path = secrets
        .write("api.header", b"Authorization: Bearer t\n")
        .unwrap();

    // Assert
    assert_eq!(path, secrets.path("api.header"));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "Authorization: Bearer t\n"
    );
    let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&path), 0o600);
    assert_eq!(mode(&root.path().join("github")), 0o700);
}
