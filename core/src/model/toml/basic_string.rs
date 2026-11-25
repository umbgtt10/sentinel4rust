// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The inside of a basic string, escapes decoded.

use crate::model::toml::toml_error::TomlError;
use alloc::format;
use alloc::string::String;
use core::str::CharIndices;

/// Reads the inside of a basic string, after its opening quote; a control character other than a tab must be
/// escaped.
#[derive(Debug, Clone, Copy, Default)]
pub struct BasicString;

impl BasicString {
    /// The decoded value and the bytes consumed, closing quote included.
    pub fn read(&self, text: &str, line_no: usize) -> Result<(String, usize), TomlError> {
        let mut chars = text.char_indices();
        let mut out = String::new();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => return Ok((out, i + 1)),
                '\\' => {
                    let (decoded, rest) = Self::escape(chars, line_no)?;
                    out.push(decoded);
                    chars = rest;
                }
                c if c != '\t' && c.is_ascii_control() => {
                    return Err(Self::error(
                        line_no,
                        format!("unescaped control character `U+{:04X}`", u32::from(c)),
                    ));
                }
                c => out.push(c),
            }
        }
        Err(Self::error(line_no, String::from("unterminated string")))
    }

    fn error(line: usize, message: String) -> TomlError {
        TomlError { line, message }
    }

    /// The character the escape after a backslash stands for, and the characters after it.
    fn escape(
        mut chars: CharIndices<'_>,
        line_no: usize,
    ) -> Result<(char, CharIndices<'_>), TomlError> {
        let (_, escape) = chars
            .next()
            .ok_or_else(|| Self::error(line_no, String::from("unterminated escape")))?;
        let decoded = match escape {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '"' => '"',
            '\\' => '\\',
            'u' => return Self::unicode(chars, line_no),
            other => return Err(Self::error(line_no, format!("invalid escape `\\{other}`"))),
        };
        Ok((decoded, chars))
    }

    /// The `\uXXXX` escape whose digits come next, and the characters after them.
    fn unicode(
        mut chars: CharIndices<'_>,
        line_no: usize,
    ) -> Result<(char, CharIndices<'_>), TomlError> {
        let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .map(|decoded| (decoded, chars))
            .ok_or_else(|| Self::error(line_no, format!("invalid escape `\\u{hex}`")))
    }
}
