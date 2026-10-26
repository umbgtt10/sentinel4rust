// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The inside of a basic string, escapes decoded.

use crate::model::toml::toml_error::TomlError;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Reads the inside of a basic string, after its opening quote.
#[derive(Debug, Clone, Copy, Default)]
pub struct BasicString;

impl BasicString {
    /// The decoded value and the bytes consumed, closing quote included.
    pub fn read(&self, text: &str, line_no: usize) -> Result<(String, usize), TomlError> {
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let mut out = String::new();
        let mut at = 0;
        while let Some(&(i, c)) = chars.get(at) {
            match c {
                '"' => return Ok((out, i + 1)),
                '\\' => {
                    let (decoded, next) = Self::escape(&chars, at + 1, line_no)?;
                    out.push(decoded);
                    at = next;
                }
                c => {
                    out.push(c);
                    at += 1;
                }
            }
        }
        Err(Self::error(line_no, String::from("unterminated string")))
    }

    fn error(line: usize, message: String) -> TomlError {
        TomlError { line, message }
    }

    /// The character the escape at `at` stands for, and the index after it.
    fn escape(
        chars: &[(usize, char)],
        at: usize,
        line_no: usize,
    ) -> Result<(char, usize), TomlError> {
        let &(_, escape) = chars
            .get(at)
            .ok_or_else(|| Self::error(line_no, String::from("unterminated escape")))?;
        let decoded = match escape {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '"' => '"',
            '\\' => '\\',
            'u' => return Self::unicode(chars, at + 1, line_no),
            other => return Err(Self::error(line_no, format!("invalid escape `\\{other}`"))),
        };
        Ok((decoded, at + 1))
    }

    /// The `\uXXXX` escape whose digits start at `at`.
    fn unicode(
        chars: &[(usize, char)],
        at: usize,
        line_no: usize,
    ) -> Result<(char, usize), TomlError> {
        let hex: String = chars.iter().skip(at).take(4).map(|(_, c)| *c).collect();
        let consumed = hex.chars().count();
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .map(|decoded| (decoded, at + consumed))
            .ok_or_else(|| Self::error(line_no, format!("invalid escape `\\u{hex}`")))
    }
}
