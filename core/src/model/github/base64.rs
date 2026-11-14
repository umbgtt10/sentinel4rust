// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Base64 (RFC 4648): the standard alphabet with padding, as HTTP Basic authorization uses it, and the URL- and
//! filename-safe one without, as JWTs use it.

use alloc::string::String;

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64, in either alphabet.
#[derive(Debug, Clone, Copy, Default)]
pub struct Base64;

impl Base64 {
    /// `bytes` in the standard alphabet, padded with `=` to a multiple of four.
    pub fn standard(&self, bytes: &[u8]) -> String {
        let mut text = Self::encode(STANDARD, bytes);
        while !text.len().is_multiple_of(4) {
            text.push('=');
        }
        text
    }

    /// `bytes` in the URL alphabet, unpadded.
    pub fn url(&self, bytes: &[u8]) -> String {
        Self::encode(URL, bytes)
    }

    fn encode(alphabet: &[u8; 64], bytes: &[u8]) -> String {
        let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let group = chunk.iter().enumerate().fold(0u32, |acc, (i, byte)| {
                acc | u32::from(*byte) << (16 - 8 * i)
            });
            for sextet in 0..=chunk.len() {
                let index = (group >> (18 - 6 * sextet)) & 0x3f;
                text.push(char::from(alphabet[index as usize]));
            }
        }
        text
    }
}
