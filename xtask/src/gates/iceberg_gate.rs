// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-iceberg4rust";
const OFFENDERS_FOUND: i32 = 2;

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
            return Err(GateFailure::NotInstalled(String::from(BINARY)));
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
            Some(OFFENDERS_FOUND) => Err(GateFailure::Found(format!(
                "a file is at or above the ceiling of {}",
                self.threshold
            ))),
            code => Err(GateFailure::Exited(code)),
        }
    }
}
