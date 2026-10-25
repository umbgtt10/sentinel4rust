// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! cargo-mutants, run in a checkout.

use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use crate::model::command::Command;
use crate::model::mutation_run::MutationRun;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::run_outcome::RunOutcome;
use crate::model::work_dir::WorkDir;
use crate::system::mutants::outcome_interpreter::OutcomeInterpreter;
use crate::traits::command_runner::CommandRunner;
use crate::traits::mutation_runner::MutationRunner;

const VERSION_TIMEOUT: Duration = Duration::from_secs(60);

/// cargo-mutants, run through a [`CommandRunner`].
pub struct CargoMutantsCli {
    runner: Rc<dyn CommandRunner>,
    work_dir: WorkDir,
    cargo: String,
    interpreter: OutcomeInterpreter,
}

impl CargoMutantsCli {
    /// Runs `cargo` (or a stand-in, for tests), writes output where `work_dir` says and reads it with
    /// `interpreter`.
    pub fn new(
        runner: Rc<dyn CommandRunner>,
        work_dir: WorkDir,
        cargo: String,
        interpreter: OutcomeInterpreter,
    ) -> Self {
        Self {
            runner,
            work_dir,
            cargo,
            interpreter,
        }
    }

    /// The arguments given to cargo, `--output` excluded (it is machine-specific).
    pub fn arguments(jobs: u32) -> Vec<String> {
        vec![
            "mutants".to_string(),
            "--jobs".to_string(),
            jobs.to_string(),
        ]
    }

    /// `cargo mutants --version` as installed, read in the work directory; `None` when it cannot be read.
    pub fn installed_version(&self) -> Option<String> {
        self.version(&self.cargo, &["mutants", "--version"], self.work_dir.root())
    }

    /// Removes the temporary files a killed run left behind.
    pub fn prune(&self) {
        let _ = fs::remove_dir_all(self.work_dir.tmp_root());
    }

    fn create_all(dirs: [&Path; 2]) -> Result<(), String> {
        for dir in dirs {
            fs::create_dir_all(dir)
                .map_err(|error| format!("create {}: {error}", dir.display()))?;
        }
        Ok(())
    }

    fn version(&self, program: &str, args: &[&str], dir: &Path) -> Option<String> {
        self.runner
            .run(&Command::new(program, args, dir, VERSION_TIMEOUT))
            .success()
            .map(|s| s.trim().to_string())
    }
}

impl MutationRunner for CargoMutantsCli {
    fn run(
        &self,
        repo: &OnboardedRepo,
        checkout: &Path,
        jobs: u32,
        timeout: Duration,
    ) -> MutationRun {
        let out = self.work_dir.mutants_out(repo);
        let tmp = self.work_dir.tmp(repo);
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&tmp);
        let unknown = || "unknown".to_string();
        let cargo_mutants_version = self
            .version(&self.cargo, &["mutants", "--version"], checkout)
            .unwrap_or_else(unknown);
        let rustc_version = self
            .version("rustc", &["--version"], checkout)
            .unwrap_or_else(unknown);
        let arguments = Self::arguments(jobs);
        let outcome = if let Err(reason) = Self::create_all([&out, &tmp]) {
            RunOutcome::ToolFailed {
                reason,
                log_tail: String::new(),
            }
        } else {
            let out_text = out.to_string_lossy().into_owned();
            let tmp_text = tmp.to_string_lossy().into_owned();
            let mut args: Vec<&str> = arguments.iter().map(String::as_str).collect();
            args.extend(["--output", &out_text]);
            let command = Command::new(&self.cargo, &args, checkout, timeout)
                .with_env("CARGO_TERM_COLOR", "never")
                .with_env("NO_COLOR", "1")
                .with_env("TMPDIR", &tmp_text);
            self.interpreter
                .interpret(&self.runner.run(&command), &out.join("mutants.out"))
        };
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&tmp);
        MutationRun {
            outcome,
            cargo_mutants_version,
            rustc_version,
            arguments: arguments.join(" "),
        }
    }
}
