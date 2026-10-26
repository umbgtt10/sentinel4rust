# A technology-free core and a cli crate

Status: **in progress** -- one commit per slice, in the order below.

## Why

Mutation testing pays for every mutant with a build and a test run, and today each mutant of this repository pays
for the whole package's suite: real git on bare repositories, `sh`, `kill`, the binary itself driven with a stub
cargo-mutants, a one-second sleep and a lock tried up to 100,000 times -- some 27 seconds of tests per mutant, for
685 mutants. Yet the logic -- the patrol and its back-off, the roster, the record kept on `mutants`, what git and
cargo-mutants answer -- needs none of it: it already talks to the world through traits. So that logic moves into a
crate that touches no technology at all, and mutation testing is scoped to that crate alone.

## The shape

The workspace grows a third member:

| member | package | library | what it holds |
|---|---|---|---|
| `core/` | `sentinel4rust-core` | `sentinel4rust` | `#![no_std]`, `#![forbid(unsafe_code)]`, no dependency; `model/`, `patrol/`, `traits/`, and `protocol/`: what git, cargo-mutants and the run unit answer, and what it means |
| `cli/` | `sentinel4rust` | `sentinel4rust_cli` | the `sentinel4rust` binary, the command line, the composition root `EntryPoint`, and every implementation: processes, git, cargo-mutants, the run unit, the clock, sleeping, stdout, the work directory's paths |
| `xtask/` | `xtask` | `xtask` | the gates |

`cli` depends on `core`, never the reverse. Both share one workspace version, unchanged.

### Ports

The ports already exist in `core/src/traits/` and stay there: `Clock`, `CommandRunner`, `Journal`,
`MutationRunner`, `Pause`, `RepoSource`, `ResultStore`. One is added, `MutantsOut`: the lists a cargo-mutants run
left in `mutants.out`, read on disk by cli's `MutantsOutDir` and in memory by the core's tests. Paths are text in
the core -- `Command`'s directory, the roster's `work_dir`, the checkout a `RepoSource` hands a `MutationRunner` --
which it carries and never treats as paths; cli turns them into `Path`s at the edge. Every one comes from UTF-8 text
(the command line, the roster, `job.toml`), so nothing is lost.

The binary's behaviour does not change: stdout, stderr and exit codes, the files a `--run-job` writes and the files
a `--once` publishes on `mutants` are compared against a build of the tree before the split after every slice that
touches them.

### The gate

A stage-2 gate, first in the run, keeps the core technology-free: its manifest may name no crate in any section, its
crate root must carry `#![no_std]` and `#![forbid(unsafe_code)]`, and no file under `core/src` or `core/tests` may
reach `std::{env,fs,io,net,os,process,thread}`, the printing macros, the file-system methods or `extern crate std`.

### Mutation testing

`.cargo/mutants.toml` scopes cargo-mutants to `core/src`; each mutant runs the core's tests only.

## Slices

| # | slice | status |
|---|---|---|
| S0 | this record | done |
| S1 | `DryGate`: one run over the workspace, with the trees it must not judge excluded by full path | done |
| S2 | the split: `cli` takes `main.rs` and all of `invocation/`; core is renamed; one workspace version; the core's root forbids unsafe code; `deploy/` installs from `cli` | done |
| S3 | paths as text in the core: `Command`, `Roster`, `RepoSource::checkout`, `MutationRunner` | |
| S4 | the `MutantsOut` port: `MutantsOutLists` and `OutcomeInterpreter` read through it; `MutantsOutDir` reads the disk | |
| S5 | the move: ten files that only read text become the core's `protocol/`; the rest of `system/` and `WorkDir` go to cli | |
| S6 | `#![no_std]`: `alloc` and `core` imports; `Schedule` on a `BTreeMap` | |
| S7 | the technology-free gate, first in stage 2 | |
| S8 | `.cargo/mutants.toml` | |
| S9 | docs, and this record moves to `docs/history/` | |

## Tests that move

No test is deleted, and no assertion is weakened. Tests whose subject moves go with it, names unchanged:

| from (`core/tests/`) | to |
|---|---|
| `invocation/` (19) and `main_tests.rs` (10, 3 of them for the end-to-end gate) | `cli/tests/` |
| `system/git/` but `outside_paths`, `remote_listing`, `unreachable_classifier` (45); `system/mutants/cargo_mutants_cli_tests.rs` (11); `system/process/` (12); `system/unit/job_runner_tests.rs`, `unit_command_runner_tests.rs` (4); `model/work_dir_tests.rs` (10) | `cli/tests/system/` |
| `system/git/{outside_paths, remote_listing, unreachable_classifier}_tests.rs`, `system/mutants/{log_tail, mutants_out_lists, outcome_interpreter, unreadable_list}_tests.rs`, `system/unit/{job_error, job_toml, result_toml}_tests.rs` (24) | `core/tests/protocol/` |

With paths as text, the tests that compared a command's directory or a checkout with a path compare the same value
as text, or as `Path::new(...)` of it, which compares the same components. `MutantsOutLists`' tests arrange their
lists in memory instead of a temporary directory, the same texts; the two that read the lists real runs wrote
include the unchanged files under `core/tests/fixtures/mutants_out/`.
