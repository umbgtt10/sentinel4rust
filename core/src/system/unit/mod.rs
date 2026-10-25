// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! cargo-mutants run in a systemd unit of its own, without the deploy keys (`ADR-DeployKeyAndRuleset`).
//!
//! The service writes the command as a job file, starts `<unit>@run.service` with `systemctl start --wait`, and
//! reads back the result and output the unit's `sentinel4rust --run-job` wrote beside it.

pub mod job_error;
pub mod job_runner;
pub mod job_toml;
pub mod result_toml;
pub mod unit_command_runner;
