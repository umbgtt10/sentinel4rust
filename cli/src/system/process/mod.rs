// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! System implementations of the small contracts: processes, the clock, sleeping, the journal.

pub mod process_group;
pub mod stdout_journal;
pub mod stream_reader;
pub mod system_clock;
pub mod system_command_runner;
pub mod thread_pause;
