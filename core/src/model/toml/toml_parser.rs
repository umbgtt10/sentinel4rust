// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Parses the TOML subset.

use crate::model::toml::basic_string::BasicString;
use crate::model::toml::comment_stripper::CommentStripper;
use crate::model::toml::document::Document;
use crate::model::toml::document_builder::DocumentBuilder;
use crate::model::toml::toml_error::TomlError;
use crate::model::toml::value::Value;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

/// Parses the TOML subset: comments, bare and quoted keys, basic and literal strings, integers, booleans and
/// arrays of tables. Anything else is rejected with the line number.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlParser;

impl TomlParser {
    /// Parses a document.
    pub fn parse(&self, text: &str) -> Result<Document, TomlError> {
        let mut builder = DocumentBuilder::new();
        for (index, raw) in text.lines().enumerate() {
            let line_no = index + 1;
            let line = CommentStripper.strip(raw, line_no)?.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("[[") {
                builder.start_array(Self::array_name(rest, line_no)?, line_no);
            } else if line.starts_with('[') {
                return Err(Self::error(
                    line_no,
                    "only `[[array]]` tables are supported",
                ));
            } else {
                let (key, value) = Self::pair(line, line_no)?;
                builder.insert(&key, value, line_no)?;
            }
        }
        Ok(builder.finish())
    }

    /// The name of a `[[name]]` header, after its opening brackets.
    fn array_name(rest: &str, line_no: usize) -> Result<String, TomlError> {
        let name = rest
            .strip_suffix("]]")
            .ok_or_else(|| Self::error(line_no, "unterminated `[[` header"))?
            .trim();
        if Self::is_bare_key(name) {
            Ok(name.to_string())
        } else {
            Err(Self::error(line_no, format!("invalid table name `{name}`")))
        }
    }

    fn error(line: usize, message: impl Into<String>) -> TomlError {
        TomlError {
            line,
            message: message.into(),
        }
    }

    fn expect_end(rest: &str, line_no: usize) -> Result<(), TomlError> {
        if rest.trim().is_empty() {
            Ok(())
        } else {
            Err(Self::error(
                line_no,
                format!("unexpected `{}`", rest.trim()),
            ))
        }
    }

    fn is_bare_key(key: &str) -> bool {
        !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    /// `key = value`, the key bare or quoted.
    fn pair(line: &str, line_no: usize) -> Result<(String, Value), TomlError> {
        let (key, rest) = if let Some(rest) = line.strip_prefix('"') {
            let (key, consumed) = BasicString.read(rest, line_no)?;
            (key, rest[consumed..].trim_start())
        } else {
            let end = line
                .find(|c: char| c == '=' || c.is_whitespace())
                .ok_or_else(|| Self::error(line_no, "expected `key = value`"))?;
            let key = &line[..end];
            if !Self::is_bare_key(key) {
                return Err(Self::error(line_no, format!("invalid key `{key}`")));
            }
            (key.to_string(), line[end..].trim_start())
        };
        let value_text = rest
            .strip_prefix('=')
            .ok_or_else(|| Self::error(line_no, "expected `=`"))?
            .trim();
        Ok((key, Self::value(value_text, line_no)?))
    }

    fn value(text: &str, line_no: usize) -> Result<Value, TomlError> {
        if let Some(rest) = text.strip_prefix('"') {
            let (s, consumed) = BasicString.read(rest, line_no)?;
            Self::expect_end(&rest[consumed..], line_no)?;
            return Ok(Value::String(s));
        }
        if let Some(rest) = text.strip_prefix('\'') {
            let end = rest
                .find('\'')
                .ok_or_else(|| Self::error(line_no, "unterminated string"))?;
            Self::expect_end(&rest[end + 1..], line_no)?;
            return Ok(Value::String(rest[..end].to_string()));
        }
        match text {
            "true" => return Ok(Value::Boolean(true)),
            "false" => return Ok(Value::Boolean(false)),
            _ => {}
        }
        let digits: String = text.chars().filter(|c| *c != '_').collect();
        digits
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| Self::error(line_no, format!("unsupported value `{text}`")))
    }
}
