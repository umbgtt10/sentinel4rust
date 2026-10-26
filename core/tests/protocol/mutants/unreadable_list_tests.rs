// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::protocol::mutants::unreadable_list::UnreadableList;

#[test]
fn to_string_names_the_list_and_why() {
    // Arrange
    let error = UnreadableList {
        name: String::from("caught.txt"),
        reason: String::from("No such file or directory (os error 2)"),
    };

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(
        text,
        "mutants.out/caught.txt unreadable: No such file or directory (os error 2)"
    );
}
