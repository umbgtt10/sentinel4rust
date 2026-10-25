// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `MutationRunner` by shelling out to `cargo mutants` and reading `mutants.out` (design 2.6).
//!
//! The lists are read from `mutants.out/{caught,missed,timeout,unviable}.txt`, which cargo-mutants writes one
//! mutant per line as `file:line:col: description`. The exit code only decides between completed, baseline
//! failed and tool failed: `3` (timeouts) takes precedence over `2` (missed), so it cannot give the counts.

use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use crate::model::counts::MutantCounts;
use crate::model::mutation_run::MutationRun;
use crate::model::roster::OnboardedRepo;
use crate::model::run_outcome::RunOutcome;
use crate::traits::{Command, CommandResult, CommandRunner, MutationRunner};

/// Lines kept from a failing run's output.
pub const LOG_TAIL_LINES: usize = 200;
/// Bytes kept from a failing run's output.
pub const LOG_TAIL_BYTES: usize = 16 * 1024;

const VERSION_TIMEOUT: Duration = Duration::from_secs(60);

/// cargo-mutants, run through a [`CommandRunner`].
pub struct CargoMutantsCli {
    runner: Rc<dyn CommandRunner>,
    out_root: PathBuf,
    cargo: String,
}

impl CargoMutantsCli {
    /// Runs `cargo` (or a stand-in, for tests) and writes output under `out_root/<repo name>`.
    pub fn new(runner: Rc<dyn CommandRunner>, out_root: PathBuf, cargo: String) -> Self {
        Self {
            runner,
            out_root,
            cargo,
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

    fn version(&self, program: &str, args: &[&str], dir: &Path) -> String {
        self.runner
            .run(&Command::new(program, args, dir, VERSION_TIMEOUT))
            .success()
            .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
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
        let out = self.out_root.join(&repo.name);
        let _ = fs::remove_dir_all(&out);
        let cargo_mutants_version = self.version(&self.cargo, &["mutants", "--version"], checkout);
        let rustc_version = self.version("rustc", &["--version"], checkout);
        let arguments = Self::arguments(jobs);
        let outcome = if let Err(error) = fs::create_dir_all(&out) {
            RunOutcome::ToolFailed {
                reason: format!("create {}: {error}", out.display()),
                log_tail: String::new(),
            }
        } else {
            let mut command = Command::new(&self.cargo, &[], checkout, timeout)
                .env("CARGO_TERM_COLOR", "never")
                .env("NO_COLOR", "1");
            command.args.clone_from(&arguments);
            command.args.push("--output".to_string());
            command.args.push(out.to_string_lossy().into_owned());
            interpret(&self.runner.run(&command), &out.join("mutants.out"))
        };
        let _ = fs::remove_dir_all(&out);
        MutationRun {
            outcome,
            cargo_mutants_version,
            rustc_version,
            arguments: arguments.join(" "),
        }
    }
}

/// Turns the process result and `mutants.out` into an outcome (the exit-code table of design 2.6).
pub fn interpret(result: &CommandResult, mutants_out: &Path) -> RunOutcome {
    match result {
        CommandResult::FailedToStart(message) => RunOutcome::ToolFailed {
            reason: format!("could not start cargo-mutants: {message}"),
            log_tail: String::new(),
        },
        CommandResult::TimedOut { stdout, stderr } => RunOutcome::ToolFailed {
            reason: "run cap hit".to_string(),
            log_tail: log_tail(stdout, stderr),
        },
        CommandResult::Exited {
            code,
            stdout,
            stderr,
        } => match code {
            Some(0 | 2 | 3) => {
                read_lists(mutants_out).unwrap_or_else(|reason| RunOutcome::ToolFailed {
                    reason,
                    log_tail: log_tail(stdout, stderr),
                })
            }
            Some(4) => RunOutcome::BaselineFailed {
                log_tail: log_tail(stdout, stderr),
            },
            Some(code) => RunOutcome::ToolFailed {
                reason: format!("cargo-mutants exited with {code}"),
                log_tail: log_tail(stdout, stderr),
            },
            None => RunOutcome::ToolFailed {
                reason: "cargo-mutants was killed by a signal".to_string(),
                log_tail: log_tail(stdout, stderr),
            },
        },
    }
}

fn read_list(dir: &Path, name: &str, required: bool) -> Result<Vec<String>, String> {
    match fs::read_to_string(dir.join(name)) {
        Ok(text) => Ok(text
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(ToString::to_string)
            .collect()),
        Err(_) if !required => Ok(Vec::new()),
        Err(error) => Err(format!("mutants.out/{name} unreadable: {error}")),
    }
}

fn read_lists(dir: &Path) -> Result<RunOutcome, String> {
    let caught = read_list(dir, "caught.txt", true)?;
    let missed = read_list(dir, "missed.txt", true)?;
    let timed_out = read_list(dir, "timeout.txt", false)?;
    let unviable = read_list(dir, "unviable.txt", false)?;
    let count = |list: &[String]| u64::try_from(list.len()).unwrap_or(u64::MAX);
    Ok(RunOutcome::Completed {
        counts: MutantCounts {
            caught: count(&caught),
            missed: count(&missed),
            timeout: count(&timed_out),
            unviable: count(&unviable),
        },
        missed,
        timed_out,
    })
}

/// The last [`LOG_TAIL_LINES`] lines and at most [`LOG_TAIL_BYTES`] bytes of a run's output.
pub fn log_tail(stdout: &str, stderr: &str) -> String {
    let combined = match (stdout.trim_end().is_empty(), stderr.trim_end().is_empty()) {
        (true, _) => stderr.trim_end().to_string(),
        (false, true) => stdout.trim_end().to_string(),
        (false, false) => format!("{}\n{}", stdout.trim_end(), stderr.trim_end()),
    };
    let lines: Vec<&str> = combined.lines().collect();
    let tail = lines[lines.len().saturating_sub(LOG_TAIL_LINES)..].join("\n");
    if tail.len() <= LOG_TAIL_BYTES {
        return tail;
    }
    let mut start = tail.len() - LOG_TAIL_BYTES;
    while !tail.is_char_boundary(start) {
        start += 1;
    }
    tail[start..].to_string()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    type Script = Box<dyn Fn(&Command) -> CommandResult>;

    struct FakeRunner {
        calls: RefCell<Vec<Command>>,
        script: Script,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, command: &Command) -> CommandResult {
            self.calls.borrow_mut().push(command.clone());
            (self.script)(command)
        }
    }

    fn exited(code: i32, stdout: &str) -> CommandResult {
        CommandResult::Exited {
            code: Some(code),
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    fn temp(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sentinel-mut-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn repo() -> OnboardedRepo {
        OnboardedRepo {
            name: "r".to_string(),
            url: "u".to_string(),
            branch: "main".to_string(),
            jobs: 3,
        }
    }

    /// A fake cargo that writes `mutants.out` with the given lists and exits with `code`.
    fn cli_with(
        label: &str,
        code: i32,
        lists: &'static [(&'static str, &'static str)],
    ) -> (CargoMutantsCli, Rc<FakeRunner>, PathBuf) {
        let root = temp(label);
        let runner = Rc::new(FakeRunner {
            calls: RefCell::new(Vec::new()),
            script: Box::new(move |c: &Command| {
                if c.args.iter().any(|a| a == "--version") {
                    return exited(0, &format!("{} 9.9\n", c.program));
                }
                let out = PathBuf::from(c.args.last().unwrap()).join("mutants.out");
                fs::create_dir_all(&out).unwrap();
                for (name, text) in lists {
                    fs::write(out.join(name), text).unwrap();
                }
                exited(code, "log line\n")
            }),
        });
        let cli = CargoMutantsCli::new(runner.clone(), root.join("out"), "cargo".to_string());
        (cli, runner, root)
    }

    #[test]
    fn runs_cargo_mutants_with_jobs_output_and_deadline_then_cleans_up() {
        let (cli, runner, root) = cli_with(
            "args",
            2,
            &[
                ("caught.txt", "src/a.rs:1:1: c1\nsrc/a.rs:2:1: c2\n"),
                ("missed.txt", "src/a.rs:3:1: m1\n"),
                ("unviable.txt", "src/a.rs:4:1: u1\n"),
            ],
        );
        let checkout = root.join("checkout");
        let run = cli.run(&repo(), &checkout, 3, Duration::from_secs(99));
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].args, ["mutants", "--version"]);
        assert_eq!(calls[1].program, "rustc");
        let main = &calls[2];
        assert_eq!(main.dir, checkout);
        assert_eq!(main.timeout, Duration::from_secs(99));
        let out = root.join("out").join("r");
        assert_eq!(
            main.args,
            ["mutants", "--jobs", "3", "--output", &out.to_string_lossy()]
        );
        assert_eq!(run.arguments, "mutants --jobs 3");
        assert_eq!(run.cargo_mutants_version, "cargo 9.9");
        assert_eq!(run.rustc_version, "rustc 9.9");
        assert_eq!(
            run.outcome,
            RunOutcome::Completed {
                counts: MutantCounts {
                    caught: 2,
                    missed: 1,
                    timeout: 0,
                    unviable: 1
                },
                missed: vec!["src/a.rs:3:1: m1".to_string()],
                timed_out: vec![],
            }
        );
        assert!(!out.exists(), "output left behind");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn exit_three_reads_timeouts_as_well() {
        let (cli, _, root) = cli_with(
            "timeouts",
            3,
            &[
                ("caught.txt", ""),
                ("missed.txt", "src/a.rs:3:1: m1\n"),
                ("timeout.txt", "src/a.rs:5:1: t1\n"),
            ],
        );
        let run = cli.run(&repo(), &root, 1, Duration::from_secs(1));
        assert!(matches!(
            run.outcome,
            RunOutcome::Completed { counts: MutantCounts { missed: 1, timeout: 1, .. }, ref timed_out, .. }
                if timed_out == &["src/a.rs:5:1: t1".to_string()]
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_completed_exit_without_lists_is_a_tool_failure() {
        let (cli, _, root) = cli_with("nolists", 0, &[]);
        let run = cli.run(&repo(), &root, 1, Duration::from_secs(1));
        assert!(
            matches!(run.outcome, RunOutcome::ToolFailed { ref reason, .. } if reason.contains("caught.txt"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_exit_code_table() {
        let none = Path::new("/nonexistent");
        assert!(matches!(
            interpret(&exited(4, "build failed"), none),
            RunOutcome::BaselineFailed { ref log_tail } if log_tail == "build failed"
        ));
        for code in [1, 5, 6, 70] {
            assert!(matches!(
                interpret(&exited(code, ""), none),
                RunOutcome::ToolFailed { ref reason, .. } if reason == &format!("cargo-mutants exited with {code}")
            ));
        }
        let killed = CommandResult::Exited {
            code: None,
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(
            matches!(interpret(&killed, none), RunOutcome::ToolFailed { ref reason, .. } if reason.contains("signal"))
        );
        let timed_out = CommandResult::TimedOut {
            stdout: "a".to_string(),
            stderr: "b".to_string(),
        };
        assert_eq!(
            interpret(&timed_out, none),
            RunOutcome::ToolFailed {
                reason: "run cap hit".to_string(),
                log_tail: "a\nb".to_string()
            }
        );
        assert!(matches!(
            interpret(&CommandResult::FailedToStart("no cargo".to_string()), none),
            RunOutcome::ToolFailed { ref reason, .. } if reason.contains("no cargo")
        ));
    }

    #[test]
    fn log_tail_keeps_the_last_lines_and_bytes() {
        let many: String = (0..500).fold(String::new(), |mut acc, i| {
            use std::fmt::Write as _;
            let _ = writeln!(acc, "line {i}");
            acc
        });
        let tail = log_tail(&many, "");
        assert_eq!(tail.lines().count(), LOG_TAIL_LINES);
        assert!(tail.ends_with("line 499"));
        assert!(tail.starts_with("line 300"));
        let long = "é".repeat(LOG_TAIL_BYTES);
        let tail = log_tail(&long, "");
        assert!(tail.len() <= LOG_TAIL_BYTES);
        assert_eq!(log_tail("", "err\n"), "err");
        assert_eq!(log_tail("out\n", "err\n"), "out\nerr");
    }
}
