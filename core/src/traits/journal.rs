// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Where the one line per visit goes.

/// Where the one line per visit goes.
pub trait Journal {
    /// Writes one line.
    fn line(&self, text: &str);
}
