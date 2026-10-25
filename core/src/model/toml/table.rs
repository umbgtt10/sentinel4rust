// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A table of key-value pairs.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use crate::model::toml::toml_error::TomlError;
use crate::model::toml::value::Value;

/// A table of key-value pairs, with the line it starts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    entries: BTreeMap<String, Value>,
    line: usize,
}

impl Table {
    /// An empty table starting on `line`.
    pub fn new(line: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            line,
        }
    }

    /// Adds `key`; returns `false`, keeping the first value, if it is already there.
    pub fn insert(&mut self, key: String, value: Value) -> bool {
        match self.entries.entry(key) {
            Entry::Vacant(vacant) => {
                vacant.insert(value);
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    /// An optional integer.
    pub fn integer(&self, key: &str) -> Result<Option<i64>, TomlError> {
        self.typed(key, "an integer", |value| match value {
            Value::Integer(integer) => Some(*integer),
            _ => None,
        })
    }

    /// The keys present.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// The line the table starts on.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Rejects keys outside `allowed`, to catch typos.
    pub fn only(&self, allowed: &[&str]) -> Result<(), TomlError> {
        match self.keys().find(|key| !allowed.contains(key)) {
            Some(key) => Err(TomlError {
                line: self.line,
                message: format!("unknown key `{key}`"),
            }),
            None => Ok(()),
        }
    }

    /// A required integer.
    pub fn required_integer(&self, key: &str) -> Result<i64, TomlError> {
        self.required(key, self.integer(key)?)
    }

    /// A required string.
    pub fn required_string(&self, key: &str) -> Result<String, TomlError> {
        self.required(key, self.string(key)?)
    }

    /// An optional string.
    pub fn string(&self, key: &str) -> Result<Option<String>, TomlError> {
        self.typed(key, "a string", |value| match value {
            Value::String(string) => Some(string.clone()),
            _ => None,
        })
    }

    fn required<T>(&self, key: &str, value: Option<T>) -> Result<T, TomlError> {
        value.ok_or_else(|| TomlError {
            line: self.line,
            message: format!("`{key}` is missing"),
        })
    }

    fn typed<T>(
        &self,
        key: &str,
        kind: &str,
        read: impl Fn(&Value) -> Option<T>,
    ) -> Result<Option<T>, TomlError> {
        match self.entries.get(key) {
            None => Ok(None),
            Some(value) => read(value).map(Some).ok_or_else(|| TomlError {
                line: self.line,
                message: format!("`{key}` must be {kind}"),
            }),
        }
    }
}
