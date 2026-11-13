// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A cargo-mutants run read: its exit code and `mutants.out` (`ADR-ResultsFailuresAndBackOff`).
//!
//! The lists are read from `mutants.out/{caught,missed,timeout,unviable}.txt`, which cargo-mutants writes one
//! mutant per line as `file:line:col: description`. The exit code only decides between completed, baseline
//! failed and tool failed: `3` (timeouts) takes precedence over `2` (missed), so it cannot give the counts. The test
//! timeout cargo-mutants set from the baseline is read from `mutants.out/debug.log`, the one place it states it.

pub mod log_tail;
pub mod mutants_out_lists;
pub mod outcome_interpreter;
pub mod test_timeout_line;
pub mod unreadable_list;
