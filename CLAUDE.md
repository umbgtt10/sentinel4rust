# sentinel4rust

## Meaning

`sentinel4rust` hunts mutants around the clock. It runs on a dedicated machine, walks a roster of repositories, and
whenever a repository's `main` moves it runs full mutation testing with cargo-mutants against that exact commit and
publishes the surviving mutants on the repository's own orphan `mutants` branch.

It is not a gate. Nothing waits for it, and it never touches `main`. It answers one question per commit, after the
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

A workspace of two members, with a virtual root that holds no package:

| member | what it is |
|---|---|
| `core/` | package `sentinel4rust`: the library and the `sentinel4rust` binary. No dependencies. |
| `xtask/` | gate orchestration, run as `cargo xtask`. `publish = false`. |

`core/src/` is `invocation/` (the command line, `EntryPoint`), `traits/`, `model/` (`branch/`, `onboarding/`,
`toml/`), `patrol/` (the loop) and `system/` (`process/`, `git/`, `mutants/`); `core/tests/` mirrors it. `deploy/`
holds the systemd unit, the set-up notes, the access check and the example roster.

## Quality Gates

### Mandatory after every change to any member's `src/` or `tests/`

`just stage1`
`just stage2`

If either gate is not green, the work is not complete.

Stage 1 is `cargo fmt`, clippy pedantic with `-D warnings` on every target, and the tests -- cargo built-ins only.
Stage 2 is `cargo xtask stage2`, the gates in this order, all on, nothing baselined:

| gate | asks |
|---|---|
| stern4rust | do the house coding rules hold (both packages) |
| `cargo shear` | is every declared dependency used, in the section that uses it |
| `cargo dry4rust` | did this change add duplicated code to `core/src` (floor 25) |
| `cargo crap4rust` | is any function complex and untested (threshold 15) |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk at or above 7 |
| end to end | do the `#[ignore]`d tests pass against the real cargo-mutants 27.1.0 |

The stern gate runs stern4rust pinned at commit `8b0e8e5` of `https://github.com/umbgtt10/stern4rust` (a constant
in `xtask/src/gates/stern_gate.rs`), installed repo-locally under `target/stern4rust`, never over the global
`cargo-stern4rust`: the gate installs it whenever `target/stern4rust/bin/cargo-stern4rust` is missing or
`target/stern4rust/REV` does not name the pin, locally and on CI alike. All its rules hold, so there is no stern
baseline. `docs/header.txt` holds the header every `.rs` file carries, and `stern4rust.toml` names it.

`dry4rust-baseline.json` records no group and may never gain one. Re-record it only to drop a group that is gone, at
the same floor: `cargo dry4rust --path core/src --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline`.

The iceberg ceiling is a ratchet just above the worst file: lower it when the score improves, never raise it to turn
a red build green.

`cargo install just`, `cargo install cargo-dry4rust --locked`, `cargo install cargo-crap4rust`,
`cargo install cargo-twin4rust`, `cargo install cargo-iceberg4rust`, `cargo install --locked cargo-shear --version
1.14.0`, `cargo install cargo-llvm-cov` (crap4rust's coverage), `cargo install --locked cargo-mutants --version
27.1.0` (the end-to-end gate refuses any other version). `.github/workflows/ci.yml` runs both stages on Linux.

### Branch protection

`.github/rulesets/main.json` is the copy of record for `main`'s ruleset: a pull request with both CI jobs green,
repository admins bypassing. It has not been applied; applying it is a server-side change for the owner to make
with `gh api repos/umbgtt10/sentinel4rust/rulesets --method POST --input .github/rulesets/main.json`, and an update
after that is a `PUT` to the ruleset's id.

## This tool's rules

- `GitCli` pushes `refs/heads/mutants` and nothing else. Tests assert `main` is byte-identical after every publish.
- The core has no dependencies: keep it that way unless a dependency removes real code. TOML goes through
  `core/src/model/toml/`. The dev-only `xtask` uses `serde` and `serde_json`, from `[workspace.dependencies]`.
- Every external effect goes through a trait in `core/src/traits/`; `Sentinel` gets them all through
  `Dependencies`, and `EntryPoint::compose` builds the real ones. New behaviour in the loop is tested with the fake
  in `core/tests/support/world.rs`; git parts against `core/tests/support/scripted_runner.rs`.
- Git tests use bare repositories and `file://` remotes under a temporary directory (`core/tests/support`); no
  network.
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
