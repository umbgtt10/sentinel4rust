// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-mutants";

// The tests that drive the real cargo-mutants, `#[ignore]`d in stage 1 because
// they need it installed. They are pinned to one version: the exit codes and
// the `mutants.out` lists they check are that version's.
pub struct EndToEndGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
    version: String,
}

impl<'a> EndToEndGate<'a> {
    pub fn new(runner: &'a dyn CommandRunner, manifest_path: String, version: String) -> Self {
        Self {
            runner,
            manifest_path,
            version,
        }
    }

    fn pinned(&self) -> Result<(), GateFailure> {
        let outcome = self.runner.run_capturing(
            "cargo",
            &[String::from("mutants"), String::from("--version")],
        )?;
        let installed = outcome.stdout.trim();
        if outcome.exit_code == Some(0)
            && installed
                .split_whitespace()
                .any(|word| word == self.version)
        {
            Ok(())
        } else {
            Err(GateFailure::NotInstalledAt {
                binary: String::from(BINARY),
                version: self.version.clone(),
            })
        }
    }
}

impl Gate for EndToEndGate<'_> {
    fn label(&self) -> String {
        String::from("End to end (real cargo-mutants)")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalledAt {
                binary: String::from(BINARY),
                version: self.version.clone(),
            });
        }
        self.pinned()?;

        let args = vec![
            String::from("test"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
            String::from("--workspace"),
            String::from("--"),
            String::from("--ignored"),
        ];
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(_) => Err(GateFailure::Found(String::from(
                "an end-to-end test against the real cargo-mutants failed",
            ))),
            None => Err(GateFailure::Exited(None)),
        }
    }
}
