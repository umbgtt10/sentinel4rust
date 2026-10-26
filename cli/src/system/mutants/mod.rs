// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `MutationRunner` by shelling out to `cargo mutants` and reading `mutants.out` (`ADR-ResultsFailuresAndBackOff`);
//! what the run means is the core's `protocol::mutants`.

pub mod cargo_mutants_cli;
pub mod mutants_out_dir;
