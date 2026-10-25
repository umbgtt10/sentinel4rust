// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The command line and wiring (design 3, 5).

use std::fs::{self, File};
use std::path::PathBuf;
use std::rc::Rc;

use crate::git::GitCli;
use crate::model::onboarding::roster::Roster;
use crate::model::onboarding::toml_roster::TomlRoster;
use crate::model::round_step::RoundStep;
use crate::mutants::CargoMutantsCli;
use crate::sentinel::{Dependencies, Sentinel, startup_line};
use crate::system::{StdoutJournal, SystemClock, SystemCommandRunner, ThreadPause};
use crate::traits::{Clock, CommandRunner};

/// Usage text.
pub const USAGE: &str = "\
usage: sentinel4rust --roster <file> [--once]

  --roster <file>  the roster (TOML); see docs/DESIGN.md
  --once           visit every repository once and exit (1 if any visit failed)
  --help           this text
  --version        the version
";

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    /// Run with this roster.
    Run {
        /// The roster file.
        roster: PathBuf,
        /// One pass and exit.
        once: bool,
    },
    /// Print usage.
    Help,
    /// Print the version.
    Version,
}

/// Parses the arguments after the program name.
pub fn parse_args(args: &[String]) -> Result<Invocation, String> {
    let mut roster = None;
    let mut once = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Invocation::Help),
            "--version" | "-V" => return Ok(Invocation::Version),
            "--once" => once = true,
            "--roster" => {
                let value = iter.next().ok_or("--roster needs a file")?;
                roster = Some(PathBuf::from(value));
            }
            other => {
                if let Some(value) = other.strip_prefix("--roster=") {
                    roster = Some(PathBuf::from(value));
                } else {
                    return Err(format!("unknown argument `{other}`"));
                }
            }
        }
    }
    let roster = roster.ok_or("--roster is required")?;
    Ok(Invocation::Run { roster, once })
}

/// Holds the work directory's lock for the life of the process (design 5).
pub struct WorkDirLock {
    _file: File,
}

impl WorkDirLock {
    /// Takes the lock, or fails if another instance holds it.
    pub fn acquire(roster: &Roster) -> Result<Self, String> {
        fs::create_dir_all(&roster.work_dir)
            .map_err(|e| format!("create {}: {e}", roster.work_dir.display()))?;
        let path = roster.work_dir.join("sentinel4rust.lock");
        let file = File::create(&path).map_err(|e| format!("open {}: {e}", path.display()))?;
        file.try_lock().map_err(|_| {
            format!(
                "another sentinel4rust is running on {} (lock {})",
                roster.work_dir.display(),
                path.display()
            )
        })?;
        Ok(Self { _file: file })
    }
}

/// Whether a `--once` pass counts as failed.
pub fn failed(step: &RoundStep) -> bool {
    matches!(
        step,
        RoundStep::BackedOff { .. } | RoundStep::Unreachable { .. }
    )
}

/// Reads the roster, takes the lock, wires everything and runs. Returns the process exit code.
pub fn run(roster_path: &PathBuf, once: bool) -> Result<i32, String> {
    let text = fs::read_to_string(roster_path)
        .map_err(|e| format!("read {}: {e}", roster_path.display()))?;
    let roster = TomlRoster.parse(&text).map_err(|e| e.to_string())?;
    let _lock = WorkDirLock::acquire(&roster)?;
    let runner: Rc<dyn CommandRunner> = Rc::new(SystemCommandRunner);
    let git = Rc::new(GitCli::new(runner.clone(), roster.work_dir.clone()));
    git.prune(&roster.repos);
    let mutants = CargoMutantsCli::new(runner, roster.work_dir.join("out"), "cargo".to_string());
    println!("{}", startup_line(&roster, SystemClock.now()));
    let mut sentinel = Sentinel::new(
        roster,
        Dependencies {
            source: Box::new(git.clone()),
            runner: Box::new(mutants),
            store: Box::new(git),
            clock: Box::new(SystemClock),
            pause: Box::new(ThreadPause),
            journal: Box::new(StdoutJournal),
        },
    );
    if once {
        let steps = sentinel.run_once();
        Ok(i32::from(steps.iter().any(failed)))
    } else {
        sentinel.run_forever()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn parses_roster_and_once() {
        assert_eq!(
            parse_args(&args(&["--roster", "r.toml", "--once"])),
            Ok(Invocation::Run {
                roster: PathBuf::from("r.toml"),
                once: true
            })
        );
        assert_eq!(
            parse_args(&args(&["--roster=r.toml"])),
            Ok(Invocation::Run {
                roster: PathBuf::from("r.toml"),
                once: false
            })
        );
        assert_eq!(parse_args(&args(&["--help"])), Ok(Invocation::Help));
        assert_eq!(parse_args(&args(&["-V"])), Ok(Invocation::Version));
    }

    #[test]
    fn rejects_bad_command_lines() {
        assert!(parse_args(&args(&[])).is_err());
        assert!(parse_args(&args(&["--roster"])).is_err());
        assert!(parse_args(&args(&["--roster", "r", "--frobnicate"])).is_err());
    }

    #[test]
    fn a_second_lock_on_the_same_work_dir_fails() {
        let dir = std::env::temp_dir().join(format!("sentinel-lock-{}", std::process::id()));
        let roster = TomlRoster
            .parse(&format!(
                "work_dir = \"{}\"\n[[repo]]\nname = \"a\"\nurl = \"u\"",
                dir.display()
            ))
            .unwrap();
        let first = WorkDirLock::acquire(&roster).unwrap();
        let error = WorkDirLock::acquire(&roster).err().unwrap();
        assert!(error.contains("another sentinel4rust"), "{error}");
        drop(first);
        assert!(WorkDirLock::acquire(&roster).is_ok());
        let _ = fs::remove_dir_all(dir);
    }
}
