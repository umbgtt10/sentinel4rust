# sentinel4rust

## Meaning

`sentinel4rust` hunts mutants around the clock. It runs on a dedicated machine, finds the repositories that ask to be
tested -- through a read-only GitHub App, every one with a `.cargo/mutants.toml` -- or walks a roster of them, and
whenever a repository's `main` moves it runs full mutation testing with cargo-mutants against that exact commit and
publishes the surviving mutants in the repository's folder of one shared results repository, `umbgtt10/mutants`.

It is not a gate. Nothing waits for it, and it never writes to the repositories it watches. It answers one question per commit, after the
fact: which changes to this code would its tests not notice?

`docs/ARCHITECTURE.md` maps the code; `docs/ADRs/` holds the load-bearing decisions; `docs/OPEN_POINTS.md` the known
gaps; the original design is in `docs/history/`, unchanged and not kept up to date.

It is self-contained.

## Boundary Rule

This repository is **SELF-CONTAINED**.

The LLM **SHALL NOT cross its boundaries without asking**.

That means:
- do not inspect, edit, or rely on files outside `sentinel4rust/` unless the user explicitly asks
- do not pull assumptions from sibling repositories or crates
- do not propose cross-repository changes by default

The repository is public: nothing private about the repositories it watches belongs in it.

## Layout

A workspace of three members, with a virtual root that holds no package:

| member | what it is |
|---|---|
| `core/` | package `sentinel4rust-core`, library `sentinel4rust`: the patrol, the roster, the record in the results repository, and what git, cargo-mutants and the run unit answer. `#![no_std]`, no dependency, no technology. |
| `cli/` | package `sentinel4rust`: the `sentinel4rust` binary and a library, `sentinel4rust_cli` -- the command line, `EntryPoint` and every implementation of the core's contracts. Depends on the core only. |
| `xtask/` | gate orchestration, run as `cargo xtask`. `publish = false`. |

`core/src/` is `traits/`, `model/` (`branch/`, `github/`, `onboarding/`, `toml/`), `patrol/` (the loop and discovery)
and `protocol/` (`git/`, `mutants/`, `unit/`: what the tools answer, read as text); `cli/src/` is `invocation/` (the
command line, `EntryPoint`, `GithubAccess`) and `system/` (`process/`, `git/`, `github/`, `mutants/`, `unit/` and
`work_dir.rs`); each `tests/` mirrors its
`src/`. The binary is installed with `cargo install --path cli --locked`. `deploy/` holds the systemd unit, the set-up
notes, the access check and the example roster.

## Quality Gates

### Mandatory after every change to any member's `src/` or `tests/`

`just stage1`
`just stage2`

If either gate is not green, the work is not complete.

Stage 1 is `cargo fmt`, clippy pedantic with `-D warnings` on every target, and the tests -- cargo built-ins only.
Stage 2 is `cargo xtask stage2`, the gates in this order, all on, nothing baselined:

| gate | asks |
|---|---|
| technology-free core | does the core stay `no_std`, dependency-free and free of technology |
| stern4rust | do the house coding rules hold (every member) |
| `cargo shear` | is every declared dependency used, in the section that uses it |
| `cargo dry4rust` | did this change add duplicated code to `core/src` or `cli/src` (floor 25) |
| `cargo crap4rust` | is any function complex and untested (threshold 15) |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk at or above 7 |
| end to end | do the `#[ignore]`d tests pass against the real cargo-mutants 27.1.0 |
| `cargo deny` | do the dependencies carry an advisory, a refused licence or a foreign source |

The gates come from serge4rust, the house's gates as a library, on crates.io (`serge4rust = "0.3.2"`): every gate,
the stage, the runner, the failures, the crap report and the commit scoring. `xtask/src` keeps `main.rs`, which sets
each gate with this repository's constants, and the gates that are this repository's own, on serge4rust's `Gate` and
`CommandRunner`: the end-to-end gate, the tool's alone, and the iceberg gate, whose arguments -- the threshold after
the packages -- and wording serge4rust's does not reproduce. A gate that needs a change is changed in serge4rust and
released, never patched in place.

`just stage1` starts with `just sync`: `cargo update`, every dependency to its
newest compatible version, in the lock file only, while the tools are in
development. It needs the network. A requirement in a manifest never moves, so a
published crate's users can still resolve what they could; what it moves is
committed on its own. Breaking versions are a separate decision, not taken here.

`[workspace.metadata.ripe4rust]` in the root `Cargo.toml` names the gates an upgrade has to pass, `just stage1` and
`just stage2`, for ripe4rust. `just ripe` runs `cargo ripe4rust plan`: which upgrades, of the toolchain or a
dependency, are ripe -- a plan, nothing applied (`cargo install cargo-ripe4rust`).

The technology-free gate runs first, because its fix -- moving code into `cli` -- changes what every gate behind it
measures, and it needs no installed tool: `sentinel4rust-core`'s manifest may name no crate in any section (read from
`cargo metadata`), `core/src/lib.rs` must carry `#![no_std]` and `#![forbid(unsafe_code)]`, and no file under
`core/src` or `core/tests` may reach `std::{env,fs,io,net,os,process,thread}`, a printing macro, a file-system method
of `Path` or `extern crate std` (read with syn, so string literals are data). It is serge4rust's boundary gate,
given this rule in `xtask/src/main.rs`. It cannot see a use inside a macro's arguments, and a method of the core's own named like
a file-system one would be reported.

The stern gate runs the release on PATH, `cargo stern4rust` 0.20.1 or later, which holds all thirty-nine rules, as
every other gate runs its tool; until that release, 2026-10-28, it installed a pinned commit of stern4rust under
`target/stern4rust` instead. All its rules hold, so there is no stern baseline. `docs/header.txt` holds the header every `.rs` file carries, and `stern4rust.toml` names it.

dry4rust makes one run from the workspace root, with `xtask/`, `core/tests/` and `cli/tests/` excluded by full path,
so a copy from one crate into the other is seen. `dry4rust-baseline.json` records no group and may never gain one.
Re-record it only to drop a group that is gone, at the same floor: `cargo dry4rust --path "$PWD" --exclude
"$PWD/xtask" --exclude "$PWD/core/tests" --exclude "$PWD/cli/tests" --min-nodes 25 --baseline
"$PWD/dry4rust-baseline.json" baseline`.

The iceberg ceiling is a ratchet just above the worst file: lower it when the score improves, never raise it to turn
a red build green.

cargo-deny runs **last**, and is the one gate that can turn red with nothing
changed: its advisories come from the RustSec database, so a newly published
one fails an untouched checkout, and it fetches that database over the network.
That is the point of it, and running last keeps it from hiding what the house
gates say about a change. `deny.toml` is the same file in every *4rust
repository and slotgate: licences MIT, Apache-2.0 and Unicode-3.0 -- the union
of what their trees carry -- crates.io as the only source, duplicate versions a
warning, and a wildcard requirement refused except a path dependency in an
unpublished crate. An accepted advisory goes in `[advisories] ignore` with its
id and its reason. The gate fetches the advisory database first, on its own and
up to three times, then checks `--frozen` -- locked and offline -- so a fetch
that keeps failing reports the gate as unable to run, never as an advisory
found. It decodes cargo-deny's exit code, one bit per failed check, into the
checks' names; a `deny.toml` it cannot read exits 1, as advisories does, and the
streamed output says which.

`cargo install just`, `cargo install cargo-dry4rust --locked`, `cargo install cargo-crap4rust`,
`cargo install cargo-twin4rust`, `cargo install cargo-iceberg4rust`, `cargo install --locked cargo-shear --version
1.14.0`, `cargo install --locked cargo-deny --version 0.20.2`, `cargo install cargo-llvm-cov` (crap4rust's coverage), `cargo install --locked cargo-mutants --version
27.1.0` (the end-to-end gate refuses any other version). `.github/workflows/ci.yml` runs both stages on Linux.

### Branch protection

`.github/rulesets/main.json` is the copy of record for `main`'s ruleset: a pull request with both CI jobs green,
repository admins bypassing. It has not been applied; applying it is a server-side change for the owner to make
with `gh api repos/umbgtt10/sentinel4rust/rulesets --method POST --input .github/rulesets/main.json`, and an update
after that is a `PUT` to the ruleset's id.

### Every commit carries core's scores

The owner's rule of 2026-11-11: every commit message carries the grip and
braintax of `core`, and their ratio, scored as the commit stages it.
`.githooks/commit-msg` runs `cargo xtask grip-braintax` on the message's file,
which writes the scores in as a paragraph of their own above the trailers:
`core`'s `grip_absolute_total` over its `total_braintax` and their ratio, under
a title naming the releases of `cargo-grip4rust` and `cargo-braintax4rust` --
scores of different releases do not compare. The files under `core/` that the
index holds are exported into a scratch directory and scored there, so nothing
unstaged counts; `cli` and `xtask` are not scored. An amend scores again, in
place of the earlier scores; a message with no words is left alone, so git
still aborts its commit; a scoring that fails stops the commit, and
`--no-verify` skips it, deliberately. Git runs a repository's own hooks only
where a clone asks it to: `just hooks`, once a clone, on each machine.
`cargo xtask grip-braintax` with no file prints the scores.

`cargo install cargo-grip4rust`
`cargo install cargo-braintax4rust`

## This tool's rules

- Only `ResultsClone` pushes, and only the results repository's `main`; nothing is ever pushed to a watched
  repository. Tests assert the watched repository's refs are byte-identical after every publish.
- The core has no dependencies: keep it that way unless a dependency removes real code. TOML goes through
  `core/src/model/toml/`. `cli` depends on the core and `serde_json`, for the GitHub API's answers
  (`ADR-NoDependencies`). The dev-only `xtask` uses `serge4rust`, from `[workspace.dependencies]`.
- The core is technology-free: `#![no_std]`, and nothing under `core/` touches the file system, a process, a thread,
  the environment or stdout. Every external effect goes through a trait in `core/src/traits/`; `Sentinel` gets them
  all through `Dependencies`, and `EntryPoint::compose`, in `cli`, builds the real ones. A new effect is a trait in
  `core/src/traits/`, a fake in `core/tests/support/` and an implementation in `cli/src/system/`, composed in
  `EntryPoint`; paths in the core are text, turned into `Path`s in `cli`.
- New behaviour in the loop is tested with the fake in `core/tests/support/world.rs`; git parts against
  `cli/tests/support/scripted_runner.rs`.
- Git tests use bare repositories and `file://` remotes under a temporary directory (`cli/tests/support`); no
  network.
- `.cargo/mutants.toml` mutates `core/src` only, each mutant against the core's own tests (`test_workspace = false`):
  what cargo-mutants and sentinel4rust's own runs test here.
- Time goes through `Clock` and `Pause`; no test reads the clock or sleeps.

## Strict Rules

### Test preservation

**NEVER remove a test without explicitly asking the user first.** If a change makes a test fail, update the
assertion to the new correct behaviour; a test moved to another file is moved, not deleted.

### Assertion integrity

**NEVER relax a test assertion without explicitly asking the user first.** Relaxing means replacing a specific
assertion with a weaker one that would pass regardless of correctness.

### Docs follow the code

Every change adds its entry under `[Unreleased]` in `CHANGELOG.md`, and keeps `docs/ARCHITECTURE.md` and
`docs/OPEN_POINTS.md` true. An ADR is a snapshot of a decision as it stands: change it when the decision changes.

## Orthogonality, trait surface and cognitive complexity

**When changing productive code, always maximize orthogonality and testable surface through traits, and minimize
cognitive complexity.**

- prefer extracting behavior behind traits so individual pieces can be tested and swapped independently
- prefer small, focused methods with a single responsibility over large methods with many branches
- prefer named structs with methods over free functions operating on external state
- when `crap4rust` or a reviewer flags a function as too complex, reduce it by extracting internal structs with
  methods and adding integration coverage — not by extracting standalone helper functions
- never increase cognitive complexity to pass a test; find the root cause and fix it there
- make constructors depend on traits, not directly on concrete implementations
- ALL dependencies are injected through the SINGLE constructor and stored in the struct
- apply the same split recursively to nested dependencies: trait first, state/data model second, concrete
  implementation third

## User coding standards

- one object per file; records where record-home puts them
- no unnecessary comments in code
- unit tests are not allowed. Only integration tests are
- consolidate scattered functions inside structs as appropriate
- no `&mut` input parameters; prefer return values
- only use `pub mod` in `mod.rs` and `lib.rs`
- split test files so there is one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- apply AAA (`Arrange`, `Act`, `Assert`) structure to tests with blank-line separation between the three sections
- use `// Arrange & Act` if there is no separate `Arrange`
- use `// Act & Assert` if there is no separate `Act`
- add the repository copyright and license header to every Rust source file
- tests should be named as follows `<method under test>_<test description>_<result>`
- do not use fully qualified paths; use `use` imports instead
