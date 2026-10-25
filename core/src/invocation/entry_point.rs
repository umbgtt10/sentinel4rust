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
use crate::system::git::git_cli::GitCli;
use crate::system::mutants::cargo_mutants_cli::CargoMutantsCli;
use crate::system::mutants::log_tail::LogTail;
use crate::system::mutants::outcome_interpreter::OutcomeInterpreter;
use crate::system::process::stdout_journal::StdoutJournal;
use crate::system::process::system_clock::SystemClock;
use crate::system::process::system_command_runner::SystemCommandRunner;
use crate::system::process::thread_pause::ThreadPause;
use crate::traits::clock::Clock;
use crate::traits::command_runner::CommandRunner;

/// Lines kept from a failing run's output.
const LOG_TAIL_LINES: usize = 200;
/// Bytes kept from a failing run's output.
const LOG_TAIL_BYTES: usize = 16 * 1024;
/// Usage text.
const USAGE: &str = "\
usage: sentinel4rust --roster <file> [--once]

  --roster <file>  the roster (TOML); see docs/DESIGN.md
  --once           visit every repository once and exit (1 if any visit failed)
  --help           this text
  --version        the version
";

/// Where every real part is built and the command line is answered.
#[derive(Debug, Clone, Default)]
pub struct EntryPoint;

impl EntryPoint {
    /// The sentinel for `roster`, wired to git, cargo-mutants, the system clock, sleeping and stdout; the
    /// leftovers of a crash are pruned first.
    pub fn compose(&self, roster: Roster) -> Sentinel {
        let runner: Rc<dyn CommandRunner> = Rc::new(SystemCommandRunner);
        let work_dir = WorkDir::new(roster.work_dir.clone());
        let git = Rc::new(GitCli::new(runner.clone(), work_dir.clone()));
        git.prune(&roster.repos);
        let mutants = CargoMutantsCli::new(
            runner,
            work_dir,
            "cargo".to_string(),
            OutcomeInterpreter::new(LogTail::new(LOG_TAIL_LINES, LOG_TAIL_BYTES)),
        );
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
            Ok(CommandLine::Run { roster, once }) => match self.start(&roster, once) {
                Ok(code) => ExitCode::from(code),
                Err(error) => {
                    eprintln!("sentinel4rust: {error}");
                    ExitCode::FAILURE
                }
            },
            Err(error) => {
                eprintln!("sentinel4rust: {error}\n\n{USAGE}");
                ExitCode::from(2)
            }
        }
    }

    fn start(&self, roster_path: &Path, once: bool) -> Result<u8, StartupError> {
        let text = fs::read_to_string(roster_path).map_err(|e| StartupError::Unreadable {
            path: roster_path.to_path_buf(),
            reason: e.to_string(),
        })?;
        let roster = TomlRoster.parse(&text)?;
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
}
