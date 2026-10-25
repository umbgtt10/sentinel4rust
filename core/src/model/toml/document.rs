// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A parsed document.

use std::collections::BTreeMap;

use crate::model::toml::table::Table;

/// A parsed document: the root table and the arrays of tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Keys before the first `[[...]]` header.
    pub root: Table,
    /// `[[name]]` sections, in order.
    pub arrays: BTreeMap<String, Vec<Table>>,
}
