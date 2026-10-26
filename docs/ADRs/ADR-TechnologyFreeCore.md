# ADR — A technology-free core

## Status

Accepted.

## Context

The service is mutation-tested like every repository it watches, and mutation testing pays for every mutant with a
build and a test run. While one package held everything, each of its 685 mutants ran that package's whole suite --
real git on bare repositories, `sh`, `kill`, the binary driven with a stub cargo-mutants, a one-second sleep, a lock
tried up to 100,000 times -- and the 136 mutants in the code that talks to processes, git, files and systemd were
mutated beside the logic. Yet the logic -- which repository to visit and when to back off, the roster, the record on
`mutants`, what git and cargo-mutants answer -- already reached the world through traits.

## Decision

The logic lives in `sentinel4rust-core` (`core/`, library `sentinel4rust`): `#![no_std]` and
`#![forbid(unsafe_code)]`, depending on no crate, declaring a trait for every effect and holding paths as text; the
`sentinel4rust` package (`cli/`) is the binary, the command line, the composition root and every implementation of
those traits, and depends on the core alone.

## Forcing constraints / Evidence

`cargo mutants --list` names 550 mutants, all in the core, where the package before the split named 685. The core's
174 tests touch no disk and start no process, and run in a hundredth of a second. Measured on two fresh copies, one
`==` turned `!=` in `CommitHash::parse` as cargo-mutants changes it, medians of three with other jobs holding the load
average at five to seven: building and running the package's tests after the edit went from 14.5 s to 0.8 s, a clean
build of its tests -- what each cargo-mutants worker pays first -- from 6.3 s to 4.1 s.

The binary's behaviour did not move: a build of the tree before the split and each step agreed on sixteen
invocations -- stdout, stderr, exit code -- and on the files a `--run-job` writes and a `--once` publishes.

## Rejected alternatives

- **One package, with `exclude_globs` on the implementations.** Scopes the mutants but not the tests each mutant runs,
  and nothing would stop the next change reaching for `std::fs` inside the patrol.
- **git's conversation in the core too.** `GitRunner`, `RecordReader` and `CrateReadiness` need only `CommandRunner`,
  but they are the git command line's knowledge; they stay with the rest of `system/git/` in `cli`, and can move once
  the core's own runs show what is worth it.
- **A path type in the core.** The core never treats a path as a path, and every one comes from UTF-8 text; `String`
  carries it, and `cli` turns it into a `Path` where it touches the disk.

## Consequences

A new effect is a trait in `core/src/traits/`, a fake in `core/tests/support/` and an implementation in
`cli/src/system/`, composed in `EntryPoint`. What git, cargo-mutants and the run unit answer is read in the core's
`protocol/`, as text; running them is `cli`'s. The binary is installed with `cargo install --path cli --locked`. A
core mutant that only a test of the implementations used to kill now survives, and wants a core test.

## Enforcement

The "Technology-free core" gate, first in stage 2: the core's manifest names no crate in any section;
`core/src/lib.rs` carries `#![no_std]` and `#![forbid(unsafe_code)]`; no file under `core/src` or `core/tests`
reaches `std::{env,fs,io,net,os,process,thread}`, a printing macro, a file-system method of `Path` or
`extern crate std`. `.cargo/mutants.toml` scopes cargo-mutants to `core/src`, each mutant tested by the core's tests
only.

## Related

- `ADR-NoDependencies.md`, `ADR-InjectedTime.md`, `ADR-PlainBinaryPackaging.md`.
- `docs/ARCHITECTURE.md`; the plan as it landed, `docs/history/[IMPLEMENTED] CORE_CLI_SPLIT.md`.
