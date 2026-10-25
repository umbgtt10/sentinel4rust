// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::system::git::remote_listing::RemoteListing;

const LISTING: &str = "\
1111111111111111111111111111111111111111\trefs/heads/main
2222222222222222222222222222222222222222\trefs/heads/mutants \n";

#[test]
fn has_a_listed_ref_returns_true() {
    // Arrange
    let listing = RemoteListing::parse(LISTING);

    // Act & Assert
    assert!(listing.has("refs/heads/main"));
    assert!(listing.has("refs/heads/mutants"));
}

#[test]
fn has_an_unlisted_ref_returns_false() {
    // Arrange
    let listing = RemoteListing::parse(LISTING);

    // Act
    let present = listing.has("refs/heads/trunk");

    // Assert
    assert!(!present);
}

#[test]
fn parse_of_lines_without_a_tab_lists_nothing() {
    // Arrange & Act
    let listing = RemoteListing::parse("refs/heads/main\n\nwarning: redirecting\n");

    // Assert
    assert!(!listing.has("refs/heads/main"));
}
