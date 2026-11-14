// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::base64_url::Base64Url;

#[test]
fn encode_of_the_rfc_4648_vectors_matches_them_without_padding() {
    // Arrange
    let vectors = [
        ("", ""),
        ("f", "Zg"),
        ("fo", "Zm8"),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg"),
        ("fooba", "Zm9vYmE"),
        ("foobar", "Zm9vYmFy"),
    ];

    // Act
    let encoded: Vec<String> = vectors
        .iter()
        .map(|(plain, _)| Base64Url.encode(plain.as_bytes()))
        .collect();

    // Assert
    let expected: Vec<&str> = vectors.iter().map(|(_, b64)| *b64).collect();
    assert_eq!(encoded, expected);
}

#[test]
fn encode_uses_the_url_alphabet() {
    // Arrange
    let bytes = [0xfb, 0xff];

    // Act
    let encoded = Base64Url.encode(&bytes);

    // Assert
    assert_eq!(encoded, "-_8");
}
