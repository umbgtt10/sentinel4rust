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
        .spawn(Some(stream), None)
        .join()
        .expect("the reader ends");

    // Assert
    assert_eq!(text, "line one\nline \u{fffd} two\n");
}

#[test]
fn spawn_on_a_stream_with_a_limit_keeps_only_its_last_bytes() {
    // Arrange
    let all: Vec<u8> = b"abcdefghijklmnopqrstuvwxyz"
        .iter()
        .copied()
        .cycle()
        .take(100_000)
        .collect();
    let stream = Cursor::new(all.clone());

    // Act
    let text = StreamReader
        .spawn(Some(stream), Some(1000))
        .join()
        .expect("the reader ends");

    // Assert
    assert_eq!(text.as_bytes(), &all[all.len() - 1000..]);
}

#[test]
fn spawn_on_no_stream_returns_nothing() {
    // Arrange
    let stream: Option<Cursor<Vec<u8>>> = None;

    // Act
    let text = StreamReader
        .spawn(stream, None)
        .join()
        .expect("the reader ends");

    // Assert
    assert_eq!(text, "");
}
