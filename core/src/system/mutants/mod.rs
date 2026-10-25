// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `MutationRunner` by shelling out to `cargo mutants` and reading `mutants.out` (design 2.6).
//!
//! The lists are read from `mutants.out/{caught,missed,timeout,unviable}.txt`, which cargo-mutants writes one
//! mutant per line as `file:line:col: description`. The exit code only decides between completed, baseline
//! failed and tool failed: `3` (timeouts) takes precedence over `2` (missed), so it cannot give the counts.

pub mod cargo_mutants_cli;
pub mod log_tail;
pub mod mutants_out_lists;
pub mod outcome_interpreter;
pub mod unreadable_list;
