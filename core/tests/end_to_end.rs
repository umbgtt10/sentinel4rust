//! End to end: the real binary, real git, `file://` remotes.
//!
//! Stage 1 puts a stub `cargo` first on `PATH` that writes a `mutants.out` like cargo-mutants does. Stage 2
//! (`cargo xtask stage2`, needs `cargo install cargo-mutants --version 27.1.0 --locked`) runs the real thing on
//! a three-function fixture crate.

mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output};

use support::{Origin, TempDir, git};

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

struct Bench {
    origin: Origin,
    work: TempDir,
    bin: TempDir,
    roster: std::path::PathBuf,
}

impl Bench {
    fn new() -> Self {
        let origin = Origin::new();
        let work = TempDir::new("e2e-work");
        let bin = TempDir::new("e2e-bin");
        let stub = bin.path().join("cargo");
        fs::write(&stub, STUB).unwrap();
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
        let roster = work.path().join("roster.toml");
        fs::write(
            &roster,
            format!(
                "work_dir = \"{}\"\npause_seconds = 1\njobs = 2\n\n[[repo]]\nname = \"fixture\"\nurl = \"{}\"\n",
                work.path().join("state").display(),
                origin.url()
            ),
        )
        .unwrap();
        Self {
            origin,
            work,
            bin,
            roster,
        }
    }

    fn once(&self, stub_exit: &str, use_stub: bool) -> Output {
        let path = if use_stub {
            format!(
                "{}:{}",
                self.bin.path().display(),
                std::env::var("PATH").unwrap()
            )
        } else {
            std::env::var("PATH").unwrap()
        };
        Command::new(env!("CARGO_BIN_EXE_sentinel4rust"))
            .args(["--roster", &self.roster.to_string_lossy(), "--once"])
            .env("PATH", path)
            .env("STUB_EXIT", stub_exit)
            .output()
            .unwrap()
    }

    fn mutants(&self, path: &str) -> String {
        git(&self.origin.bare(), &["show", &format!("mutants:{path}")])
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout(output),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn once_publishes_survivors_then_finds_main_unchanged() {
    let bench = Bench::new();
    let head = bench.origin.commit_crate();
    let main_before = bench.origin.remote_ref("refs/heads/main");

    let first = bench.once("2", true);
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

    // The run happened in the detached checkout, which is gone.
    let state = bench.work.path().join("state");
    let ran_in = fs::read_to_string(state.join("out/ran-in")).unwrap();
    assert!(ran_in.trim().ends_with("runs/fixture"), "{ran_in}");
    assert!(!state.join("runs/fixture").exists());

    let second = bench.once("2", true);
    assert_success(&second);
    assert!(
        stdout(&second).contains("step=unchanged"),
        "{}",
        stdout(&second)
    );
    assert_eq!(
        git(&bench.origin.bare(), &["rev-list", "--count", "mutants"]),
        "1"
    );
}

#[test]
fn a_red_main_publishes_baseline_failed() {
    let bench = Bench::new();
    bench.origin.commit_crate();
    let output = bench.once("4", true);
    assert_success(&output);
    assert!(
        stdout(&output).contains("outcome=baseline-failed"),
        "{}",
        stdout(&output)
    );
    let summary = bench.mutants("summary.toml");
    assert!(summary.contains("outcome = \"baseline-failed\""));
    assert!(summary.contains("baseline: test x ... FAILED"));
    let files = git(&bench.origin.bare(), &["ls-tree", "--name-only", "mutants"]);
    assert!(!files.contains("missed.txt"), "{files}");
}

#[test]
fn a_tool_failure_backs_off_and_once_exits_one() {
    let bench = Bench::new();
    bench.origin.commit_crate();
    let output = bench.once("70", true);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("step=backed-off reason=\"cargo-mutants exited with 70\""));
    assert_eq!(bench.origin.remote_ref("refs/heads/mutants"), "");
}

#[test]
fn a_repository_without_filters_is_skipped() {
    let bench = Bench::new();
    bench
        .origin
        .commit(&[("Cargo.toml", "[package]\nname = \"x\"\n")]);
    let output = bench.once("2", true);
    assert_success(&output);
    assert!(stdout(&output).contains("step=skipped reason=no-filters"));
}

#[test]
fn bad_command_lines_and_rosters_fail_cleanly() {
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_sentinel4rust"))
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(run(&[]).status.code(), Some(2));
    assert!(run(&["--help"]).status.success());
    let dir = TempDir::new("e2e-bad");
    let roster = dir.path().join("r.toml");
    fs::write(&roster, "work_dir = 1").unwrap();
    let output = run(&["--roster", &roster.to_string_lossy()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("roster"));
}

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

fn has_cargo_mutants() -> bool {
    Command::new("cargo")
        .args(["mutants", "--version"])
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
#[ignore = "stage 2: needs cargo-mutants installed"]
fn real_cargo_mutants_on_a_fixture_crate() {
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
    let output = bench.once("", false);
    assert_success(&output);
    let missed = bench.mutants("missed.txt");
    assert!(missed.contains("double"), "{missed}");
    assert!(missed.contains("is_positive"), "{missed}");
    assert!(!missed.contains("add"), "{missed}");
    // Keys carry no line numbers.
    assert!(
        missed.lines().all(|l| l.starts_with("src/lib.rs: ")),
        "{missed}"
    );
    let summary = bench.mutants("summary.toml");
    assert!(summary.contains("outcome = \"completed\""), "{summary}");
}
