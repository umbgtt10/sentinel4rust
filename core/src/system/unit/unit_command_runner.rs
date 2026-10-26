// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Commands run in the run unit, away from the deploy keys.

use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use crate::model::command::Command;
use crate::model::command_result::CommandResult;
use crate::system::unit::job_toml::JobToml;
use crate::system::unit::result_toml::ResultToml;
use crate::traits::command_runner::CommandRunner;

/// The instance of the template unit, and the job directory it reads.
const INSTANCE: &str = "run";
/// How much longer than the command's own deadline `systemctl` may wait for the unit.
const MARGIN: Duration = Duration::from_secs(10 * 60);

/// Runs each command in `<unit>@run.service`: the job is written under `jobs`, the unit started and waited for
/// through `inner`, and its result read back. One command at a time, as the sentinel runs them.
pub struct UnitCommandRunner {
    inner: Rc<dyn CommandRunner>,
    jobs: PathBuf,
    unit: String,
}

impl UnitCommandRunner {
    /// Writes jobs under `jobs` and starts `unit` through `inner`.
    pub fn new(inner: Rc<dyn CommandRunner>, jobs: PathBuf, unit: String) -> Self {
        Self { inner, jobs, unit }
    }

    fn result(dir: &Path) -> CommandResult {
        let read = |name: &str| fs::read_to_string(dir.join(name)).unwrap_or_default();
        let Ok(text) = fs::read_to_string(dir.join("result.toml")) else {
            return CommandResult::FailedToStart(format!(
                "the run unit left no result in {}",
                dir.display()
            ));
        };
        ResultToml
            .parse(&text, &read("stdout"), &read("stderr"))
            .unwrap_or_else(|error| CommandResult::FailedToStart(error.to_string()))
    }
}

impl CommandRunner for UnitCommandRunner {
    fn run(&self, command: &Command) -> CommandResult {
        let dir = self.jobs.join(INSTANCE);
        let _ = fs::remove_dir_all(&dir);
        if let Err(error) = fs::create_dir_all(&dir)
            .and_then(|()| fs::write(dir.join("job.toml"), JobToml.render(command)))
        {
            return CommandResult::FailedToStart(format!("job {}: {error}", dir.display()));
        }
        let unit = format!("{}@{INSTANCE}.service", self.unit);
        let start = Command::new(
            "systemctl",
            &["start", "--wait", &unit],
            &dir.to_string_lossy(),
            command.timeout().saturating_add(MARGIN),
        );
        let started = self.inner.run(&start);
        if started.success().is_none() {
            return CommandResult::FailedToStart(format!(
                "{}: {}",
                start.display(),
                started.describe()
            ));
        }
        Self::result(&dir)
    }
}
