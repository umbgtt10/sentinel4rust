// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::temp_dir::TempDir;
use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::model::work_dir::WorkDir;
use sentinel4rust::system::mutants::cargo_mutants_cli::CargoMutantsCli;
use sentinel4rust::system::mutants::log_tail::LogTail;
use sentinel4rust::system::mutants::outcome_interpreter::OutcomeInterpreter;
use sentinel4rust::traits::command_runner::CommandRunner;
use sentinel4rust::traits::mutation_runner::MutationRunner;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

// A fake cargo that writes `mutants.out` with the given lists and exits with `code`.
fn cli_with(
    work: &TempDir,
    code: i32,
    lists: &'static [(&'static str, &'static str)],
) -> (CargoMutantsCli, Rc<FakeRunner>) {
    let runner = Rc::new(FakeRunner {
        calls: RefCell::new(Vec::new()),
        tmp_existed: RefCell::new(Vec::new()),
        script: Box::new(move |c: &Command| {
            if c.args().iter().any(|a| a == "--version") {
                return exited(0, &format!("{} 9.9\n", c.program()));
            }
            let out = PathBuf::from(c.args().last().unwrap()).join("mutants.out");
            fs::create_dir_all(&out).unwrap();
            for (name, text) in lists {
                fs::write(out.join(name), text).unwrap();
            }
            exited(code, "log line\n")
        }),
    });
    let cli = CargoMutantsCli::new(
        runner.clone(),
        WorkDir::new(work.path().to_path_buf()),
        "cargo".to_string(),
        OutcomeInterpreter::new(LogTail::new(200, 16 * 1024)),
    );
    (cli, runner)
}

fn exited(code: i32, stdout: &str) -> CommandResult {
    CommandResult::Exited {
        code: Some(code),
        stdout: stdout.to_string(),
        stderr: String::new(),
    }
}

// Records, for every command, whether the TMPDIR it was given existed while it ran.
struct FakeRunner {
    calls: RefCell<Vec<Command>>,
    tmp_existed: RefCell<Vec<bool>>,
    script: Box<dyn Fn(&Command) -> CommandResult>,
}

impl CommandRunner for FakeRunner {
    fn run(&self, command: &Command) -> CommandResult {
        self.calls.borrow_mut().push(command.clone());
        self.tmp_existed.borrow_mut().push(
            command
                .env()
                .get("TMPDIR")
                .is_some_and(|dir| Path::new(dir).is_dir()),
        );
        (self.script)(command)
    }
}

fn repo() -> OnboardedRepo {
    OnboardedRepo {
        name: "r".to_string(),
        url: "u".to_string(),
        branch: "main".to_string(),
        jobs: 3,
        run_timeout: Duration::from_secs(3600),
    }
}

#[test]
fn arguments_of_a_job_count_ask_mutants_for_that_many_jobs() {
    // Arrange & Act
    let arguments = CargoMutantsCli::arguments(3);

    // Assert
    assert_eq!(arguments, ["mutants", "--jobs", "3"]);
}

#[test]
fn installed_version_failing_is_unknown() {
    // Arrange
    let work = TempDir::new("mutants-no-version");
    let runner = Rc::new(FakeRunner {
        calls: RefCell::new(Vec::new()),
        tmp_existed: RefCell::new(Vec::new()),
        script: Box::new(|_: &Command| exited(101, "")),
    });
    let cli = CargoMutantsCli::new(
        runner,
        WorkDir::new(work.path().to_path_buf()),
        "cargo".to_string(),
        OutcomeInterpreter::new(LogTail::new(200, 16 * 1024)),
    );

    // Act
    let version = cli.installed_version();

    // Assert
    assert_eq!(version, None);
}

#[test]
fn installed_version_reads_cargo_mutants_version_in_the_work_dir() {
    // Arrange
    let work = TempDir::new("mutants-version");
    let (cli, runner) = cli_with(&work, 0, &[]);

    // Act
    let version = cli.installed_version();

    // Assert
    assert_eq!(version, Some(String::from("cargo 9.9")));
    let calls = runner.calls.borrow();
    assert_eq!(calls[0].args(), ["mutants", "--version"]);
    assert_eq!(calls[0].dir(), work.path());
}

#[test]
fn prune_removes_leftover_temporary_files() {
    // Arrange
    let work = TempDir::new("mutants-prune");
    let leftover = work.path().join("tmp").join("r").join("build-copy");
    fs::create_dir_all(&leftover).unwrap();
    let (cli, _) = cli_with(&work, 0, &[]);

    // Act
    cli.prune();

    // Assert
    assert!(!work.path().join("tmp").exists());
}

// cargo-mutants matches --re against `file:line:col: description`, so the
// names go in whole, escaped, and anchored to match nothing else.
#[test]
fn recheck_runs_only_those_mutants_and_reads_their_lists() {
    // Arrange
    let work = TempDir::new("mutants-recheck");
    let (cli, runner) = cli_with(
        &work,
        2,
        &[
            ("caught.txt", "src/a.rs:3:1: replace + with * in f\n"),
            ("missed.txt", "src/a.rs:9:1: replace f -> bool with true\n"),
        ],
    );
    let checkout = work.path().join("checkout");
    let names = [
        "src/a.rs:3:1: replace + with * in f".to_string(),
        "src/a.rs:9:1: replace f -> bool with true".to_string(),
    ];

    // Act
    let outcome = cli.recheck(&repo(), &checkout, 2, Duration::from_secs(99), &names);

    // Assert
    let calls = runner.calls.borrow();
    assert_eq!(calls.len(), 1);
    let out = work.path().join("out").join("r");
    assert_eq!(
        calls[0].args(),
        [
            "mutants",
            "--jobs",
            "2",
            "--re",
            r"^(?:src/a\.rs:3:1: replace \+ with \* in f|src/a\.rs:9:1: replace f \-> bool with true)$",
            "--output",
            &out.to_string_lossy()
        ]
    );
    assert_eq!(calls[0].dir(), checkout);
    assert_eq!(calls[0].timeout(), Duration::from_secs(99));
    assert_eq!(
        outcome,
        RunOutcome::Completed {
            counts: MutantCounts {
                caught: 1,
                missed: 1,
                timeout: 0,
                unviable: 0
            },
            missed: vec!["src/a.rs:9:1: replace f -> bool with true".to_string()],
            timed_out: vec![],
        }
    );
    assert!(!out.exists(), "output left behind");
}

#[test]
fn run_completed_without_lists_is_a_tool_failure() {
    // Arrange
    let work = TempDir::new("mutants-nolists");
    let (cli, _) = cli_with(&work, 0, &[]);

    // Act
    let run = cli.run(&repo(), work.path(), 1, Duration::from_secs(1));

    // Assert
    assert!(
        matches!(run.outcome, RunOutcome::ToolFailed { ref reason, .. } if reason.contains("caught.txt"))
    );
}

#[test]
fn run_exiting_three_reads_the_timeouts_too() {
    // Arrange
    let work = TempDir::new("mutants-timeouts");
    let (cli, _) = cli_with(
        &work,
        3,
        &[
            ("caught.txt", ""),
            ("missed.txt", "src/a.rs:3:1: m1\n"),
            ("timeout.txt", "src/a.rs:5:1: t1\n"),
        ],
    );

    // Act
    let run = cli.run(&repo(), work.path(), 1, Duration::from_secs(1));

    // Assert
    assert!(matches!(
        run.outcome,
        RunOutcome::Completed { counts: MutantCounts { missed: 1, timeout: 1, .. }, ref timed_out, .. }
            if timed_out == &["src/a.rs:5:1: t1".to_string()]
    ));
}

// A 12-hour run's output is kept as a tail: only its end reaches the log tail.
#[test]
fn run_keeps_only_the_tail_of_cargo_mutants_output() {
    // Arrange
    let work = TempDir::new("mutants-capture");
    let (cli, runner) = cli_with(&work, 2, &[("caught.txt", ""), ("missed.txt", "")]);

    // Act
    cli.run(&repo(), work.path(), 1, Duration::from_secs(9));

    // Assert
    let calls = runner.calls.borrow();
    assert_eq!(calls[2].capture_limit(), Some(64 * 1024));
    assert_eq!(calls[0].capture_limit(), None);
}

#[test]
fn run_passes_jobs_output_and_deadline_then_cleans_up() {
    // Arrange
    let work = TempDir::new("mutants-args");
    let (cli, runner) = cli_with(
        &work,
        2,
        &[
            ("caught.txt", "src/a.rs:1:1: c1\nsrc/a.rs:2:1: c2\n"),
            ("missed.txt", "src/a.rs:3:1: m1\n"),
            ("unviable.txt", "src/a.rs:4:1: u1\n"),
        ],
    );
    let root = work.path().to_path_buf();
    let checkout = root.join("checkout");

    // Act
    let run = cli.run(&repo(), &checkout, 3, Duration::from_secs(99));

    // Assert
    let calls = runner.calls.borrow();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].args(), ["mutants", "--version"]);
    assert_eq!(calls[1].program(), "rustc");
    let main = &calls[2];
    assert_eq!(main.dir(), checkout);
    assert_eq!(main.timeout(), Duration::from_secs(99));
    let out = root.join("out").join("r");
    assert_eq!(
        main.args(),
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
}

// cargo-mutants copies the tree once per job into TMPDIR; under the work
// directory those copies are on disk, not in a RAM-backed /tmp.
#[test]
fn run_points_tmpdir_into_the_work_dir_and_removes_it_after() {
    // Arrange
    let work = TempDir::new("mutants-tmpdir");
    let (cli, runner) = cli_with(&work, 2, &[("caught.txt", ""), ("missed.txt", "")]);
    let checkout = work.path().join("checkout");

    // Act
    cli.run(&repo(), &checkout, 1, Duration::from_secs(9));

    // Assert
    let tmp = work.path().join("tmp").join("r");
    let calls = runner.calls.borrow();
    assert_eq!(
        calls[2].env().get("TMPDIR"),
        Some(&tmp.to_string_lossy().into_owned())
    );
    assert!(runner.tmp_existed.borrow()[2]);
    assert!(!tmp.exists(), "temporary files left behind");
}

// rustup says why rustc does not run in a checkout whose rust-toolchain.toml
// names a toolchain the machine lacks: a skip, not a tool failure.
#[test]
fn run_without_the_checkouts_toolchain_is_toolchain_missing_and_runs_nothing() {
    // Arrange
    let work = TempDir::new("mutants-no-toolchain");
    let runner = Rc::new(FakeRunner {
        calls: RefCell::new(Vec::new()),
        tmp_existed: RefCell::new(Vec::new()),
        script: Box::new(|c: &Command| {
            if c.program() == "rustc" {
                CommandResult::Exited {
                    code: Some(1),
                    stdout: String::new(),
                    stderr: "error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed\n"
                        .to_string(),
                }
            } else {
                exited(0, "cargo-mutants 27.1.0\n")
            }
        }),
    });
    let cli = CargoMutantsCli::new(
        runner.clone(),
        WorkDir::new(work.path().to_path_buf()),
        "cargo".to_string(),
        OutcomeInterpreter::new(LogTail::new(200, 16 * 1024)),
    );

    // Act
    let run = cli.run(&repo(), work.path(), 1, Duration::from_secs(9));

    // Assert
    assert_eq!(
        run.outcome,
        RunOutcome::ToolchainMissing {
            message:
                "exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed"
                    .to_string()
        }
    );
    assert_eq!(runner.calls.borrow().len(), 2);
    assert!(!work.path().join("out").join("r").exists());
}
