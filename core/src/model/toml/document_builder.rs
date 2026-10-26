// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A document assembled line by line.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::model::toml::document::Document;
use crate::model::toml::table::Table;
use crate::model::toml::toml_error::TomlError;
use crate::model::toml::value::Value;

/// Collects key-value pairs into the root table, or into the `[[name]]` table opened last.
#[derive(Debug)]
pub struct DocumentBuilder {
    root: Table,
    arrays: BTreeMap<String, Vec<Table>>,
    current: Option<(String, Table)>,
}

impl DocumentBuilder {
    /// An empty document; its root starts on line 1.
    pub fn new() -> Self {
        Self {
            root: Table::new(1),
            arrays: BTreeMap::new(),
            current: None,
        }
    }

    /// The document, the last `[[name]]` table included.
    pub fn finish(mut self) -> Document {
        self.close_current();
        Document {
            root: self.root,
            arrays: self.arrays,
        }
    }

    /// Adds `key` to the table being filled; a key already there is an error on `line`.
    pub fn insert(&mut self, key: &str, value: Value, line: usize) -> Result<(), TomlError> {
        let table = match self.current.as_mut() {
            Some((_, table)) => table,
            None => &mut self.root,
        };
        if table.insert(key.to_string(), value) {
            Ok(())
        } else {
            Err(TomlError {
                line,
                message: format!("duplicate key `{key}`"),
            })
        }
    }

    /// Opens a new `[[name]]` table on `line`; pairs from here on go into it.
    pub fn start_array(&mut self, name: String, line: usize) {
        self.close_current();
        self.current = Some((name, Table::new(line)));
    }

    fn close_current(&mut self) {
        if let Some((name, table)) = self.current.take() {
            self.arrays.entry(name).or_default().push(table);
        }
    }
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
