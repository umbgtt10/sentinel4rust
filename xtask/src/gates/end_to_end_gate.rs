// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serge4rust::gates::gate::Gate;
use serge4rust::gates::gate_failure::GateFailure;
use serge4rust::process::command_runner::CommandRunner;

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

    // No serge4rust failure names a version, so the pinned install says itself
    // in the gate's own words.
    fn not_installed(&self) -> GateFailure {
        GateFailure::Own {
            reason: format!(
                "{BINARY} {version} is not installed -- run: cargo install {BINARY} --version {version} --locked",
                version = self.version
            ),
        }
    }

    fn pinned(&self) -> Result<(), GateFailure> {
        let outcome = self.runner.run_capturing(
            "cargo",
            &[String::from("mutants"), String::from("--version")],
        )?;
        let installed = outcome.stdout().trim();
        if outcome.is_success()
            && installed
                .split_whitespace()
                .any(|word| word == self.version)
        {
            Ok(())
        } else {
            Err(self.not_installed())
        }
    }
}

impl Gate for EndToEndGate<'_> {
    fn label(&self) -> String {
        String::from("End to end (real cargo-mutants)")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(self.not_installed());
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
            Some(_) => Err(GateFailure::Own {
                reason: String::from("an end-to-end test against the real cargo-mutants failed"),
            }),
            None => Err(GateFailure::Own {
                reason: String::from("exit code None"),
            }),
        }
    }
}
