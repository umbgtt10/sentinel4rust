// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What a cargo-mutants run left in `mutants.out`.

use crate::system::mutants::unreadable_list::UnreadableList;

/// Reads the lists one cargo-mutants run wrote.
pub trait MutantsOut {
    /// The text of the list `name` (`caught.txt`, ...), or why it cannot be read.
    fn list(&self, name: &str) -> Result<String, UnreadableList>;
}
