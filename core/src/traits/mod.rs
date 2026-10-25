// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The contracts every part of the tool talks through (design 4). Each has a system implementation and, in
//! tests, a fake.

pub mod clock;
pub mod command_runner;
pub mod journal;
pub mod mutation_runner;
pub mod pause;
pub mod repo_source;
pub mod result_store;
