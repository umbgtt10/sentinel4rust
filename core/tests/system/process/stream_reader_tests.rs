// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::system::process::stream_reader::StreamReader;
use std::io::Cursor;

#[test]
fn spawn_on_a_stream_returns_all_of_it_lossily() {
    // Arrange
    let stream = Cursor::new(b"line one\nline \xff two\n".to_vec());

    // Act
    let text = StreamReader
        .spawn(Some(stream))
        .join()
        .expect("the reader ends");

    // Assert
    assert_eq!(text, "line one\nline \u{fffd} two\n");
}

#[test]
fn spawn_on_no_stream_returns_nothing() {
    // Arrange
    let stream: Option<Cursor<Vec<u8>>> = None;

    // Act
    let text = StreamReader.spawn(stream).join().expect("the reader ends");

    // Assert
    assert_eq!(text, "");
}
