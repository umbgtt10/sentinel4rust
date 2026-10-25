// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The gates. `cargo xtask stage1` runs on every commit; `cargo xtask stage2` adds the tests that need
//! cargo-mutants installed.

use std::process::{Command, ExitCode};

const STAGE1: &[&[&str]] = &[
    &["fmt", "--all", "--check"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ],
    &["test", "--workspace"],
];

const STAGE2: &[&[&str]] = &[&["test", "--workspace", "--", "--ignored"]];

fn main() -> ExitCode {
    let stage = std::env::args().nth(1).unwrap_or_default();
    let steps: Vec<&[&str]> = match stage.as_str() {
        "stage1" => STAGE1.to_vec(),
        "stage2" => STAGE2.to_vec(),
        "all" => STAGE1.iter().chain(STAGE2).copied().collect(),
        _ => {
            eprintln!("usage: cargo xtask <stage1|stage2|all>");
            return ExitCode::from(2);
        }
    };
    for args in steps {
        eprintln!("xtask: cargo {}", args.join(" "));
        let ok = Command::new("cargo")
            .args(args)
            .status()
            .is_ok_and(|status| status.success());
        if !ok {
            eprintln!("xtask: failed: cargo {}", args.join(" "));
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
