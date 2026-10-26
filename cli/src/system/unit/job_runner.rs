// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the run unit does: one job, its result and output written beside it.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::protocol::unit::job_error::JobError;
use sentinel4rust::protocol::unit::job_toml::JobToml;
use sentinel4rust::protocol::unit::result_toml::ResultToml;
use sentinel4rust::traits::command_runner::CommandRunner;

/// Runs the command in a job directory's `job.toml` and writes `result.toml`, `stdout` and `stderr` beside it:
/// `sentinel4rust --run-job <dir>`, the run unit's `ExecStart`.
pub struct JobRunner {
    runner: Rc<dyn CommandRunner>,
}

impl JobRunner {
    /// Runs jobs through `runner`.
    pub fn new(runner: Rc<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    /// Runs the job in `dir`; fails only when it cannot be read or its result written.
    pub fn run(&self, dir: &Path) -> Result<(), JobError> {
        let job = dir.join("job.toml");
        let text = fs::read_to_string(&job)
            .map_err(|e| JobError(format!("read {}: {e}", job.display())))?;
        let result = self.runner.run(&JobToml.parse(&text)?);
        let (stdout, stderr) = match &result {
            CommandResult::Exited { stdout, stderr, .. }
            | CommandResult::TimedOut { stdout, stderr } => (stdout.as_str(), stderr.as_str()),
            CommandResult::FailedToStart(_) => ("", ""),
        };
        for (name, contents) in [
            ("stdout", stdout.to_string()),
            ("stderr", stderr.to_string()),
            ("result.toml", ResultToml.render(&result)),
        ] {
            let path = dir.join(name);
            fs::write(&path, contents)
                .map_err(|e| JobError(format!("write {}: {e}", path.display())))?;
        }
        Ok(())
    }
}
