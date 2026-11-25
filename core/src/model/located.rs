// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A mutant as cargo-mutants names it, split at its position.

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

/// Where in its file a mutant is.
struct Position {
    pub line: u64,
    pub column: u64,
}

/// A mutant as cargo-mutants names it, `file:line:col: description`, split; ordered by file, then position.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Located {
    file: String,
    line: u64,
    column: u64,
    description: String,
}

impl Located {
    /// Splits `file:line:col: description`; a line in any other form becomes its own description.
    pub fn parse(text: &str) -> Self {
        text.match_indices(':')
            .find_map(|(colon, _)| {
                let (position, description) = Self::position(&text[colon + 1..])?;
                Some(Self {
                    file: text[..colon].to_string(),
                    line: position.line,
                    column: position.column,
                    description: description.to_string(),
                })
            })
            .unwrap_or_else(|| Self {
                file: String::new(),
                line: 0,
                column: 0,
                description: text.trim().to_string(),
            })
    }

    /// The mutant without its position: `file: description`.
    pub fn base(&self) -> String {
        if self.file.is_empty() {
            self.description.clone()
        } else {
            format!("{}: {}", self.file, self.description)
        }
    }

    /// Parses `line:col: rest`.
    fn position(text: &str) -> Option<(Position, &str)> {
        let (line, rest) = text.split_once(':')?;
        let (column, rest) = rest.split_once(':')?;
        let description = rest.strip_prefix(' ')?;
        let number = |s: &str| {
            s.bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| s.parse().ok())
                .flatten()
        };
        let position = Position {
            line: number(line)?,
            column: number(column)?,
        };
        Some((position, description))
    }
}
