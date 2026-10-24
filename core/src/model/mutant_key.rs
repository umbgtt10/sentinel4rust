//! Position-free mutant identities (design 2.3).
//!
//! cargo-mutants names a mutant `src/lib.rs:12:5: replace == with != in Parser::next`. The position changes
//! whenever a line is added above it, so the published lists drop it: the key is
//! `src/lib.rs: replace == with != in Parser::next`, with ` #2`, ` #3`… when the same key occurs more than
//! once in a file, numbered in source order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A mutant's identity without line and column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MutantKey(String);

impl MutantKey {
    /// A key from its published text.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The key's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Keys for a list of located mutant names, as cargo-mutants writes them; returned sorted.
    pub fn from_located(lines: &[String]) -> Vec<Self> {
        let mut parsed: Vec<Located> = lines.iter().map(|line| Located::parse(line)).collect();
        parsed.sort();
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        let mut keys: Vec<Self> = parsed
            .into_iter()
            .map(|located| {
                let base = located.base();
                let count = seen.entry(base.clone()).or_insert(0);
                *count += 1;
                if *count == 1 {
                    Self(base)
                } else {
                    Self(format!("{base} #{count}"))
                }
            })
            .collect();
        keys.sort();
        keys
    }
}

impl fmt::Display for MutantKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How many keys appeared and disappeared between two lists.
pub fn new_and_gone(previous: &[MutantKey], current: &[MutantKey]) -> (u64, u64) {
    let previous: BTreeSet<&MutantKey> = previous.iter().collect();
    let current: BTreeSet<&MutantKey> = current.iter().collect();
    let count = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
    (
        count(current.difference(&previous).count()),
        count(previous.difference(&current).count()),
    )
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Located {
    file: String,
    line: u64,
    column: u64,
    description: String,
}

impl Located {
    /// Splits `file:line:col: description`; a line in any other form becomes its own description.
    fn parse(text: &str) -> Self {
        let mut search = 0;
        while let Some(offset) = text[search..].find(':') {
            let colon = search + offset;
            if let Some((line, column, description)) = Self::position(&text[colon + 1..]) {
                return Self {
                    file: text[..colon].to_string(),
                    line,
                    column,
                    description: description.to_string(),
                };
            }
            search = colon + 1;
        }
        Self {
            file: String::new(),
            line: 0,
            column: 0,
            description: text.trim().to_string(),
        }
    }

    /// Parses `line:col: rest`.
    fn position(text: &str) -> Option<(u64, u64, &str)> {
        let (line, rest) = text.split_once(':')?;
        let (column, rest) = rest.split_once(':')?;
        let description = rest.strip_prefix(' ')?;
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if !digits(line) || !digits(column) {
            return None;
        }
        Some((line.parse().ok()?, column.parse().ok()?, description))
    }

    fn base(&self) -> String {
        if self.file.is_empty() {
            self.description.clone()
        } else {
            format!("{}: {}", self.file, self.description)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(lines: &[&str]) -> Vec<String> {
        let lines: Vec<String> = lines.iter().map(ToString::to_string).collect();
        MutantKey::from_located(&lines)
            .into_iter()
            .map(|k| k.as_str().to_string())
            .collect()
    }

    #[test]
    fn drops_line_and_column() {
        assert_eq!(
            keys(&["src/lib.rs:12:5: replace == with != in Parser::next"]),
            ["src/lib.rs: replace == with != in Parser::next"]
        );
    }

    #[test]
    fn keys_survive_lines_inserted_above() {
        let before = keys(&[
            "src/a.rs:3:9: replace f -> bool with true",
            "src/a.rs:8:5: replace + with - in g",
        ]);
        let after = keys(&[
            "src/a.rs:13:9: replace f -> bool with true",
            "src/a.rs:18:5: replace + with - in g",
        ]);
        assert_eq!(before, after);
    }

    #[test]
    fn duplicates_get_ordinals_in_source_order() {
        assert_eq!(
            keys(&[
                "src/a.rs:20:5: replace + with - in g",
                "src/a.rs:10:5: replace + with - in g",
                "src/b.rs:1:1: replace + with - in g",
            ]),
            [
                "src/a.rs: replace + with - in g",
                "src/a.rs: replace + with - in g #2",
                "src/b.rs: replace + with - in g",
            ]
        );
    }

    #[test]
    fn paths_with_colons_and_unrecognised_lines_are_kept() {
        assert_eq!(
            keys(&["odd:dir/x.rs:4:2: replace a with b", "  something else  "]),
            ["odd:dir/x.rs: replace a with b", "something else"]
        );
    }

    #[test]
    fn new_and_gone_are_set_differences() {
        let a = [MutantKey::new("x"), MutantKey::new("y")];
        let b = [
            MutantKey::new("y"),
            MutantKey::new("z"),
            MutantKey::new("w"),
        ];
        assert_eq!(new_and_gone(&a, &b), (2, 1));
        assert_eq!(new_and_gone(&[], &b), (3, 0));
        assert_eq!(new_and_gone(&a, &a), (0, 0));
    }
}
