// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The loop: which repository to visit next, and the in-memory back-off (design 1, 2.6).
//!
//! A repository is due when `pause` has passed since its last visit and it is not backing off. Among the due
//! repositories, the one that has waited longest goes first (ties in roster order), so a repository whose
//! `main` keeps moving cannot starve the rest. Losing this state on a restart costs at most an early visit.

pub mod dependencies;
pub mod journal_line;
pub mod next;
pub mod patrol_constants;
pub mod publication_builder;
pub mod repo_state;
pub mod schedule;
pub mod sentinel;
