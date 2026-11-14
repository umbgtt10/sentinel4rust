// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::base64::Base64;

const VECTORS: [&str; 7] = ["", "f", "fo", "foo", "foob", "fooba", "foobar"];

#[test]
fn standard_of_the_rfc_4648_vectors_matches_them_with_padding() {
    // Arrange
    let expected = [
        "", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy",
    ];

    // Act
    let encoded: Vec<String> = VECTORS
        .iter()
        .map(|plain| Base64.standard(plain.as_bytes()))
        .collect();

    // Assert
    assert_eq!(encoded, expected);
    assert_eq!(Base64.standard(&[0xfb, 0xff]), "+/8=");
}

#[test]
fn url_of_the_rfc_4648_vectors_matches_them_without_padding() {
    // Arrange
    let expected = ["", "Zg", "Zm8", "Zm9v", "Zm9vYg", "Zm9vYmE", "Zm9vYmFy"];

    // Act
    let encoded: Vec<String> = VECTORS
        .iter()
        .map(|plain| Base64.url(plain.as_bytes()))
        .collect();

    // Assert
    assert_eq!(encoded, expected);
    assert_eq!(Base64.url(&[0xfb, 0xff]), "-_8");
}
