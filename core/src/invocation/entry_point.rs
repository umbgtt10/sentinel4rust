// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The composition root: the command line in, an exit code out.

use std::fs;
use std::io;
use std::path::Path;
use std::process::ExitCode;
use std::rc::Rc;

use crate::invocation::command_line::CommandLine;
use crate::invocation::startup_error::StartupError;
use crate::invocation::work_dir_lock::WorkDirLock;
use crate::model::onboarding::roster::Roster;
use crate::model::onboarding::toml_roster::TomlRoster;
use crate::model::round_step::RoundStep;
use crate::model::work_dir::WorkDir;
use crate::patrol::dependencies::Dependencies;
use crate::patrol::journal_line::JournalLine;
use crate::patrol::sentinel::Sentinel;
use crate::patrol::status_report::StatusReport;
use crate::system::git::git_cli::GitCli;
use crate::system::mutants::cargo_mutants_cli::CargoMutantsCli;
use crate::system::mutants::log_tail::LogTail;
use crate::system::mutants::outcome_interpreter::OutcomeInterpreter;
use crate::system::process::stdout_journal::StdoutJournal;
use crate::system::process::system_clock::SystemClock;
use crate::system::process::system_command_runner::SystemCommandRunner;
use crate::system::process::thread_pause::ThreadPause;
use crate::system::unit::job_runner::JobRunner;
use crate::system::unit::unit_command_runner::UnitCommandRunner;
use crate::traits::clock::Clock;
use crate::traits::command_runner::CommandRunner;

/// Lines kept from a failing run's output.
const LOG_TAIL_LINES: usize = 200;
/// Bytes kept from a failing run's output.
const LOG_TAIL_BYTES: usize = 16 * 1024;
/// Usage text.
const USAGE: &str = "\
usage: sentinel4rust --roster <file> [--once]

  --roster <file>  the roster (TOML); see deploy/roster.example.toml
  --once           visit every repository once and exit (1 if any visit failed)
  --status         fetch only, and say how far each repository's record is behind main
                   (1 if one has waited longer than stale_after_hours, or could not be judged)
  --run-job <dir>  run the job in <dir> and write its result there (the run unit's ExecStart)
  --help           this text
  --version        the version
";

/// Where every real part is built and the command line is answered.
#[derive(Debug, Clone, Default)]
pub struct EntryPoint;

impl EntryPoint {
    /// The sentinel for `roster`, wired to git, cargo-mutants (in the roster's run unit, when it names one), the
    /// system clock, sleeping and stdout; the
    /// leftovers of a crash -- checkouts, staging areas, stale worktree entries, temporary files -- are pruned
    /// first.
    pub fn compose(&self, roster: Roster) -> Sentinel {
        let runner: Rc<dyn CommandRunner> = Rc::new(SystemCommandRunner);
        let work_dir = WorkDir::new(roster.work_dir.clone());
        let git = Rc::new(GitCli::new(runner.clone(), work_dir.clone()));
        git.prune(&roster.repos);
        let mutants_runner: Rc<dyn CommandRunner> = match &roster.run_unit {
            Some(unit) => Rc::new(UnitCommandRunner::new(
                runner,
                work_dir.jobs(),
                unit.clone(),
            )),
            None => runner,
        };
        let mutants = CargoMutantsCli::new(
            mutants_runner,
            work_dir,
            "cargo".to_string(),
            OutcomeInterpreter::new(LogTail::new(LOG_TAIL_LINES, LOG_TAIL_BYTES)),
        );
        mutants.prune();
        let cargo_mutants_version = mutants.installed_version();
        Sentinel::new(
            roster,
            Dependencies {
                source: git.clone(),
                runner: Rc::new(mutants),
                store: git,
                clock: Rc::new(SystemClock),
                pause: Rc::new(ThreadPause),
                journal: Rc::new(StdoutJournal::new(Box::new(io::stdout()))),
            },
            cargo_mutants_version,
        )
    }

    /// Answers the arguments after the program name: usage, the version, or the service; 2 for a bad
    /// command line, 1 when it cannot start or a `--once` visit failed.
    pub fn run(&self, args: &[String]) -> ExitCode {
        match CommandLine::parse(args) {
            Ok(CommandLine::Help) => {
                print!("{USAGE}");
                ExitCode::SUCCESS
            }
            Ok(CommandLine::Version) => {
                println!("sentinel4rust {}", env!("CARGO_PKG_VERSION"));
                ExitCode::SUCCESS
            }
            Ok(CommandLine::Run { roster, once }) => Self::exit(self.start(&roster, once)),
            Ok(CommandLine::Status { roster }) => Self::exit(Self::status(&roster)),
            Ok(CommandLine::RunJob { dir }) => Self::exit(Self::run_job(&dir)),
            Err(error) => {
                eprintln!("sentinel4rust: {error}\n\n{USAGE}");
                ExitCode::from(2)
            }
        }
    }

    fn exit(code: Result<u8, StartupError>) -> ExitCode {
        match code {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("sentinel4rust: {error}");
                ExitCode::FAILURE
            }
        }
    }

    fn roster(roster_path: &Path) -> Result<Roster, StartupError> {
        let text = fs::read_to_string(roster_path).map_err(|e| StartupError::Unreadable {
            path: roster_path.to_path_buf(),
            reason: e.to_string(),
        })?;
        Ok(TomlRoster.parse(&text)?)
    }

    fn run_job(dir: &Path) -> Result<u8, StartupError> {
        JobRunner::new(Rc::new(SystemCommandRunner)).run(dir)?;
        Ok(0)
    }

    fn start(&self, roster_path: &Path, once: bool) -> Result<u8, StartupError> {
        let roster = Self::roster(roster_path)?;
        let _lock = WorkDirLock::acquire(&roster.work_dir)?;
        println!("{}", JournalLine.startup(&roster, SystemClock.now()));
        let mut sentinel = self.compose(roster);
        if once {
            let steps = sentinel.run_once();
            Ok(u8::from(steps.iter().any(RoundStep::is_failure)))
        } else {
            sentinel.run_forever()
        }
    }

    // Its own clones and lock under `<work_dir>/status`, so it can run beside the service.
    fn status(roster_path: &Path) -> Result<u8, StartupError> {
        let roster = Self::roster(roster_path)?;
        let status_dir = roster.work_dir.join("status");
        let _lock = WorkDirLock::acquire(&status_dir)?;
        let runner: Rc<dyn CommandRunner> = Rc::new(SystemCommandRunner);
        let report = StatusReport::new(
            Rc::new(GitCli::new(runner, WorkDir::new(status_dir))),
            Rc::new(SystemClock),
            roster.config_version,
            roster.stale_after,
        );
        let lines = report.check(&roster.repos);
        for line in &lines {
            println!("{}", line.text);
        }
        Ok(u8::from(lines.iter().any(|line| line.needs_attention)))
    }
}
