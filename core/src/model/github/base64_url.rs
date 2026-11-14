// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Base64 in the URL- and filename-safe alphabet, without padding (RFC 4648 section 5), as JWTs use it.

use alloc::string::String;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64url without padding.
#[derive(Debug, Clone, Copy, Default)]
pub struct Base64Url;

impl Base64Url {
    /// `bytes`, encoded.
    pub fn encode(&self, bytes: &[u8]) -> String {
        let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let group = chunk.iter().enumerate().fold(0u32, |acc, (i, byte)| {
                acc | u32::from(*byte) << (16 - 8 * i)
            });
            for sextet in 0..=chunk.len() {
                let index = (group >> (18 - 6 * sextet)) & 0x3f;
                text.push(char::from(ALPHABET[index as usize]));
            }
        }
        text
    }
}
