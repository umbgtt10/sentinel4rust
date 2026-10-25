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

struct FakeRunner {
    calls: RefCell<Vec<Command>>,
    script: Box<dyn Fn(&Command) -> CommandResult>,
}

impl CommandRunner for FakeRunner {
    fn run(&self, command: &Command) -> CommandResult {
        self.calls.borrow_mut().push(command.clone());
        (self.script)(command)
    }
}

fn repo() -> OnboardedRepo {
    OnboardedRepo {
        name: "r".to_string(),
        url: "u".to_string(),
        branch: "main".to_string(),
        jobs: 3,
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
