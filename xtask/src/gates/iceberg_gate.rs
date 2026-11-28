// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serge4rust::gates::gate::Gate;
use serge4rust::gates::gate_failure::GateFailure;
use serge4rust::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-iceberg4rust";
const OFFENDERS_FOUND: i32 = 2;

// This repository's own, not serge4rust's: serge4rust's passes the threshold
// before the packages and says an unexpected exit in other words, and the
// migration onto serge4rust changed no gate's arguments or output.
pub struct IcebergGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
    packages: Vec<String>,
    threshold: String,
}

impl<'a> IcebergGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        manifest_path: String,
        packages: Vec<String>,
        threshold: String,
    ) -> Self {
        Self {
            runner,
            manifest_path,
            packages,
            threshold,
        }
    }
}

impl Gate for IcebergGate<'_> {
    fn label(&self) -> String {
        String::from("File risk")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        let mut args = vec![
            String::from("iceberg4rust"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        // A virtual workspace has no root package, so the packages to analyse
        // have to be named: without them the tool cannot pick a default.
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }
        args.push(String::from("--threshold"));
        args.push(self.threshold.clone());

        // 2 is the tool's own "offenders found"; anything else non-zero is a
        // failure to run at all.
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(OFFENDERS_FOUND) => Err(GateFailure::AboveCeiling {
                ceiling: self.threshold.clone(),
            }),
            code => Err(GateFailure::Own {
                reason: format!("exit code {code:?}"),
            }),
        }
    }
}
