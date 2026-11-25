// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::protocol::git::outside_paths::OutsidePaths;

#[test]
fn escapes_from_a_nested_manifest_counts_from_its_directory() {
    // Arrange
    let text = "[dependencies]\nb = { path = \"../b\" }\n";
    let far = "[dependencies]\nx = { path = \"../../../x\" }\n";

    // Act & Assert
    assert!(!OutsidePaths.escapes("crates/a/Cargo.toml", text));
    assert!(OutsidePaths.escapes("crates/a/Cargo.toml", far));
}

#[test]
fn escapes_with_a_key_ending_in_a_second_path_is_false() {
    // Arrange
    let text = "[dependencies]\nx = { pathpath = \"../x\" }\n";

    // Act
    let escapes = OutsidePaths.escapes("Cargo.toml", text);

    // Assert
    assert!(!escapes);
}

#[test]
fn escapes_with_a_path_down_and_back_up_is_false() {
    // Arrange
    let text = "[dependencies]\nb = { path = \"crates/a/../b\" }\n";

    // Act
    let escapes = OutsidePaths.escapes("Cargo.toml", text);

    // Assert
    assert!(!escapes);
}

#[test]
fn escapes_with_a_second_path_on_a_line_reads_it() {
    // Arrange
    let text = "[dependencies]\na = { path = \"crates/a\" }, b = { path = \"../b\" }\n";

    // Act
    let escapes = OutsidePaths.escapes("Cargo.toml", text);

    // Assert
    assert!(escapes);
}

#[test]
fn escapes_with_a_sibling_path_dependency_is_true() {
    // Arrange
    let text = "[package]\nname = \"x\"\n\n[dependencies]\nsibling = { path = \"../sibling\", version = \"1\" }\n";

    // Act
    let escapes = OutsidePaths.escapes("Cargo.toml", text);

    // Assert
    assert!(escapes);
}

#[test]
fn escapes_with_an_absolute_or_single_quoted_path_reads_them_both() {
    // Arrange
    let absolute = "[dependencies.x]\npath = \"/opt/x\"\n";
    let single = "[dependencies]\nx = { path = '../x' }\n";

    // Act & Assert
    assert!(OutsidePaths.escapes("Cargo.toml", absolute));
    assert!(OutsidePaths.escapes("Cargo.toml", single));
}

#[test]
fn escapes_with_current_directory_segments_skips_them() {
    // Arrange
    let dot = "[dependencies]\nx = { path = \"./../x\" }\n";
    let empty = "[dependencies]\nx = { path = \".//../x\" }\n";

    // Act & Assert
    assert!(OutsidePaths.escapes("Cargo.toml", dot));
    assert!(OutsidePaths.escapes("Cargo.toml", empty));
}

#[test]
fn escapes_with_paths_inside_the_repository_or_commented_out_is_false() {
    // Arrange
    let text = "\
[lib]
path = \"src/lib.rs\"

[dependencies]
a = { path = \"crates/a\" }
# old = { path = \"../old\" }
classpath = \"../not-a-path-key\"
";

    // Act
    let escapes = OutsidePaths.escapes("Cargo.toml", text);

    // Assert
    assert!(!escapes);
}
