// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// End to end: the real binary, real git, `file://` remotes. Stage 1 puts a
// stub `cargo` first on `PATH` that writes a `mutants.out` like cargo-mutants
// does; the ignored test runs the real cargo-mutants 27.1.0 on a
// three-function fixture crate, in stage 2.

use crate::support::git_shell::git;
use crate::support::origin::Origin;
use crate::support::temp_dir::TempDir;
use std::env;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

const FIXTURE_LIB: &str = "\
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn is_positive(x: i32) -> bool {
    x > 0
}

pub fn double(x: i32) -> i32 {
    x * 2
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_works() {
        assert_eq!(super::add(2, 3), 5);
        assert_eq!(super::add(-2, 3), 1);
    }

    #[test]
    fn is_positive_runs() {
        let _ = super::is_positive(1);
    }
}
";
// Its one test fails without any mutation: cargo-mutants exits 4.
const RED_LIB: &str = "\
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_is_checked_against_the_wrong_sum() {
        assert_eq!(super::add(2, 2), 5);
    }
}
";
// The `+ with *` mutant leaves `i` at 0 forever: cargo-mutants exits 3.
const SPIN_LIB: &str = "\
use std::hint::black_box;

pub fn spin(limit: u64) -> u64 {
    let mut i = 0;
    while i < limit {
        i = black_box(i + 1);
    }
    i
}

#[cfg(test)]
mod tests {
    #[test]
    fn spin_counts_to_its_limit() {
        assert_eq!(super::spin(1000), 1000);
    }
}
";
const STUB: &str = r#"#!/bin/sh
# A stand-in for cargo-mutants. Behaviour comes from $STUB_EXIT (default 2).
if [ "$1" = "mutants" ] && [ "$2" = "--version" ]; then
  echo "cargo-mutants 27.1.0 (stub)"; exit 0
fi
out=""
while [ $# -gt 0 ]; do
  if [ "$1" = "--output" ]; then out="$2"; fi
  shift
done
code="${STUB_EXIT:-2}"
if [ "$code" = "4" ]; then echo "baseline: test x ... FAILED"; exit 4; fi
mkdir -p "$out/mutants.out"
printf 'src/lib.rs:2:5: replace add -> i32 with 1\n' > "$out/mutants.out/caught.txt"
printf 'src/lib.rs:6:5: replace double -> i32 with 0\nsrc/lib.rs:4:7: replace > with < in is_positive\n' > "$out/mutants.out/missed.txt"
: > "$out/mutants.out/unviable.txt"
pwd > "$out/../ran-in"
exit "$code"
"#;

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout(output),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct Bench {
    origin: Origin,
    results: Origin,
    work: TempDir,
    bin: TempDir,
    roster: PathBuf,
}

impl Bench {
    fn new() -> Self {
        let origin = Origin::new();
        let results = Origin::new();
        let work = TempDir::new("e2e-work");
        let bin = TempDir::new("e2e-bin");
        let stub = bin.path().join("cargo");
        fs::write(&stub, STUB).unwrap();
        fs::set_permissions(&stub, Permissions::from_mode(0o755)).unwrap();
        let roster = work.path().join("roster.toml");
        fs::write(
            &roster,
            format!(
                "work_dir = \"{}\"\nresults_url = \"{}\"\npause_seconds = 1\njobs = 2\n\n[[repo]]\nname = \"fixture\"\nurl = \"{}\"\n",
                work.path().join("state").display(),
                results.url(),
                origin.url()
            ),
        )
        .unwrap();
        Self {
            origin,
            results,
            work,
            bin,
            roster,
        }
    }

    fn status(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sentinel4rust"))
            .args(["--roster", &self.roster.to_string_lossy(), "--status"])
            .output()
            .unwrap()
    }

    fn mutants(&self, path: &str) -> String {
        git(
            &self.results.bare(),
            &["show", &format!("main:fixture/{path}")],
        )
    }

    // The service runs cargo-mutants without the gates' `RUSTFLAGS=-D warnings`,
    // under which a mutant leaving a parameter unused would not build.
    fn once(&self, stub_exit: &str, use_stub: bool) -> Output {
        let path = if use_stub {
            format!(
                "{}:{}",
                self.bin.path().display(),
                env::var("PATH").unwrap()
            )
        } else {
            env::var("PATH").unwrap()
        };
        Command::new(env!("CARGO_BIN_EXE_sentinel4rust"))
            .args(["--roster", &self.roster.to_string_lossy(), "--once"])
            .env("PATH", path)
            .env("STUB_EXIT", stub_exit)
            .env_remove("RUSTFLAGS")
            .output()
            .unwrap()
    }
}

fn has_cargo_mutants() -> bool {
    Command::new("cargo")
        .args(["mutants", "--version"])
        .output()
        .is_ok_and(|o| o.status.success())
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn main_bad_command_lines_and_rosters_fail_cleanly() {
    // Arrange
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_sentinel4rust"))
            .args(args)
            .output()
            .unwrap()
    };
    let dir = TempDir::new("e2e-bad");
    let roster = dir.path().join("r.toml");
    fs::write(&roster, "work_dir = 1").unwrap();

    // Act & Assert
    assert_eq!(run(&[]).status.code(), Some(2));
    assert!(run(&["--help"]).status.success());

    // Act
    let output = run(&["--roster", &roster.to_string_lossy()]);

    // Assert
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("roster"));
}

#[test]
fn once_publishes_survivors_then_finds_main_unchanged() {
    // Arrange
    let bench = Bench::new();
    let head = bench.origin.commit_crate();
    let main_before = bench.origin.remote_ref("refs/heads/main");

    // Act
    let first = bench.once("2", true);

    // Assert
    assert_success(&first);
    assert!(
        stdout(&first).contains(&format!(
            "repo=fixture step=published hash={} outcome=completed missed=2 caught=1",
            head.short()
        )),
        "{}",
        stdout(&first)
    );
    assert_eq!(
        bench.mutants("missed.txt"),
        "src/lib.rs: replace > with < in is_positive\nsrc/lib.rs: replace double -> i32 with 0"
    );
    assert_eq!(
        bench.mutants("located/missed.txt"),
        "src/lib.rs:4:7: replace > with < in is_positive\nsrc/lib.rs:6:5: replace double -> i32 with 0"
    );
    let summary = bench.mutants("summary.toml");
    assert!(
        summary.contains(&format!("commit = \"{head}\"")),
        "{summary}"
    );
    assert!(
        summary.contains("arguments = \"mutants --jobs 2\""),
        "{summary}"
    );
    assert!(summary.contains("cargo_mutants_version = \"cargo-mutants 27.1.0 (stub)\""));
    assert_eq!(bench.origin.remote_ref("refs/heads/main"), main_before);
    assert_eq!(bench.origin.remote_ref("refs/heads/mutants"), "");
    let state = bench.work.path().join("state");
    let ran_in = fs::read_to_string(state.join("out/ran-in")).unwrap();
    assert!(ran_in.trim().ends_with("runs/fixture"), "{ran_in}");
    assert!(!state.join("runs/fixture").exists());

    // Act
    let second = bench.once("2", true);

    // Assert
    assert_success(&second);
    assert!(
        stdout(&second).contains("step=unchanged"),
        "{}",
        stdout(&second)
    );
    assert_eq!(
        git(&bench.results.bare(), &["rev-list", "--count", "main"]),
        "1"
    );
}

#[test]
#[ignore = "stage 2: needs cargo-mutants installed"]
fn once_real_cargo_mutants_on_a_fixture_crate_publishes_its_survivors() {
    // Arrange
    assert!(has_cargo_mutants(), "cargo-mutants is not installed");
    let bench = Bench::new();
    bench.origin.commit(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        ),
        (".cargo/mutants.toml", "timeout_multiplier = 3.0\n"),
        ("src/lib.rs", FIXTURE_LIB),
    ]);

    // Act
    let output = bench.once("", false);

    // Assert
    assert_success(&output);
    let missed = bench.mutants("missed.txt");
    assert!(missed.contains("double"), "{missed}");
    assert!(missed.contains("is_positive"), "{missed}");
    assert!(!missed.contains("add"), "{missed}");
    assert!(
        missed.lines().all(|l| l.starts_with("src/lib.rs: ")),
        "{missed}"
    );
    let summary = bench.mutants("summary.toml");
    assert!(summary.contains("outcome = \"completed\""), "{summary}");
}

#[test]
#[ignore = "stage 2: needs cargo-mutants installed"]
fn once_real_cargo_mutants_on_a_looping_mutant_publishes_its_timeout() {
    // Arrange
    assert!(has_cargo_mutants(), "cargo-mutants is not installed");
    let bench = Bench::new();
    bench.origin.commit(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fixture_spin\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        ),
        (
            ".cargo/mutants.toml",
            "timeout_multiplier = 1.0\nminimum_test_timeout = 2.0\n",
        ),
        ("src/lib.rs", SPIN_LIB),
    ]);

    // Act
    let output = bench.once("", false);

    // Assert
    assert_success(&output);
    assert_eq!(
        bench.mutants("timeout.txt"),
        "src/lib.rs: replace + with * in spin"
    );
    assert_eq!(bench.mutants("missed.txt"), "");
    let summary = bench.mutants("summary.toml");
    assert!(summary.contains("outcome = \"completed\""), "{summary}");
    assert!(summary.contains("caught = 6\n"), "{summary}");
    assert!(summary.contains("timeout = 1\n"), "{summary}");
    assert!(
        summary
            .lines()
            .any(|line| line == "test_timeout_seconds = 2"),
        "{summary}"
    );
}

#[test]
#[ignore = "stage 2: needs cargo-mutants installed"]
fn once_real_cargo_mutants_on_a_red_fixture_publishes_baseline_failed() {
    // Arrange
    assert!(has_cargo_mutants(), "cargo-mutants is not installed");
    let bench = Bench::new();
    bench.origin.commit(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fixture_red\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        ),
        (".cargo/mutants.toml", "timeout_multiplier = 3.0\n"),
        ("src/lib.rs", RED_LIB),
    ]);

    // Act
    let output = bench.once("", false);

    // Assert
    assert_success(&output);
    assert!(
        stdout(&output).contains("outcome=baseline-failed"),
        "{}",
        stdout(&output)
    );
    let summary = bench.mutants("summary.toml");
    assert!(
        summary.contains("outcome = \"baseline-failed\""),
        "{summary}"
    );
    assert!(
        summary.contains("cargo test failed in an unmutated tree"),
        "{summary}"
    );
    assert!(
        summary.contains("add_is_checked_against_the_wrong_sum"),
        "{summary}"
    );
    let files = git(
        &bench.results.bare(),
        &["ls-tree", "-r", "--name-only", "main"],
    );
    assert!(!files.contains("missed.txt"), "{files}");
}

#[test]
fn once_red_main_publishes_baseline_failed() {
    // Arrange
    let bench = Bench::new();
    bench.origin.commit_crate();

    // Act
    let output = bench.once("4", true);

    // Assert
    assert_success(&output);
    assert!(
        stdout(&output).contains("outcome=baseline-failed"),
        "{}",
        stdout(&output)
    );
    let summary = bench.mutants("summary.toml");
    assert!(summary.contains("outcome = \"baseline-failed\""));
    assert!(summary.contains("baseline: test x ... FAILED"));
    let files = git(
        &bench.results.bare(),
        &["ls-tree", "-r", "--name-only", "main"],
    );
    assert!(!files.contains("missed.txt"), "{files}");
}

#[test]
fn once_repository_without_filters_is_skipped() {
    // Arrange
    let bench = Bench::new();
    bench
        .origin
        .commit(&[("Cargo.toml", "[package]\nname = \"x\"\n")]);

    // Act
    let output = bench.once("2", true);

    // Assert
    assert_success(&output);
    assert!(stdout(&output).contains("step=skipped reason=no-filters"));
}

#[test]
fn once_tool_failure_backs_off_and_exits_one() {
    // Arrange
    let bench = Bench::new();
    bench.origin.commit_crate();

    // Act
    let output = bench.once("70", true);

    // Assert
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("step=backed-off reason=\"cargo-mutants exited with 70\""));
    assert_eq!(bench.results.remote_ref("refs/heads/main"), "");
}

// What the run unit's ExecStart does: the job in, the result and output out.
#[test]
fn run_job_runs_the_command_and_writes_its_result() {
    // Arrange
    let job = TempDir::new("e2e-job");
    fs::write(
        job.path().join("job.toml"),
        format!(
            "program = \"sh\"\ndir = \"{}\"\ntimeout_seconds = 10\n\n[[arg]]\nvalue = \"-c\"\n\n[[arg]]\nvalue = \"echo ran; exit 4\"\n",
            job.path().display()
        ),
    )
    .unwrap();

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_sentinel4rust"))
        .args(["--run-job", &job.path().to_string_lossy()])
        .output()
        .unwrap();

    // Assert
    assert_success(&output);
    assert_eq!(
        fs::read_to_string(job.path().join("result.toml")).unwrap(),
        "kind = \"exited\"\ncode = 4\n"
    );
    assert_eq!(
        fs::read_to_string(job.path().join("stdout")).unwrap(),
        "ran\n"
    );
}

#[test]
fn status_after_a_publish_is_current_then_behind_and_never_pushes() {
    // Arrange
    let bench = Bench::new();
    let head = bench.origin.commit_crate();
    assert_success(&bench.once("2", true));
    let results_before = bench.results.remote_ref("refs/heads/main");

    // Act
    let current = bench.status();
    let moved = bench.origin.commit(&[("later.txt", "after the publish")]);
    let behind = bench.status();

    // Assert
    assert_success(&current);
    assert!(
        stdout(&current).contains(&format!(
            "repo=fixture state=current main={} record={}",
            head.short(),
            head.short()
        )),
        "{}",
        stdout(&current)
    );
    assert_success(&behind);
    assert!(
        stdout(&behind).contains(&format!(
            "repo=fixture state=behind main={} record={} waited_s=",
            moved.short(),
            head.short()
        )),
        "{}",
        stdout(&behind)
    );
    assert_eq!(bench.results.remote_ref("refs/heads/main"), results_before);
}
