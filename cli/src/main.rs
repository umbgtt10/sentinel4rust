// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `sentinel4rust --roster <file> [--once]`.

use std::env::args;
use std::process::ExitCode;

use sentinel4rust_cli::invocation::entry_point::EntryPoint;

fn main() -> ExitCode {
    EntryPoint.run(&args().skip(1).collect::<Vec<String>>())
}
