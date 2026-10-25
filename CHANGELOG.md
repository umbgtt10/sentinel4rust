# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The first implementation (`48c8fb1`) is the service as `docs/IMPLEMENTED-FEATURES.md` describes it. Since then the
repository has been brought under the gates and the layout of the other `*4rust` tools. The service's behaviour is
unchanged: the same command line and exit codes, the same roster format, the same journal lines and the same
files and commit messages on `mutants`. The library's paths all changed; it has never been published, and nothing outside
this repository uses it.

### Added
- A checkout in which rustc does not run -- its `rust-toolchain.toml` names a toolchain the machine lacks -- is
  skipped with `toolchain-missing` and rustup's message, instead of counting as a tool failure; cargo-mutants is not
  started (open point 10, toolchains). `RunOutcome` and `SkipReason` gain a `ToolchainMissing { message }` variant.
- A repository whose manifests have a `path` dependency outside it is skipped with `path-outside-repo`, naming the
  manifest, instead of failing its baseline in a checkout that has no sibling (open point 10, path dependencies).
  `SkipReason` gains `PathOutsideRepo { manifest }` and is no longer `Copy`.
- A commit with submodules has them initialised in the run's worktree, recursively, before cargo-mutants runs (open
  point 10, the submodule part).
- Stage 2 runs the real cargo-mutants 27.1.0 on a crate with a failing baseline and on one with a mutant that loops,
  beside the survivors' fixture; the lists those runs wrote are kept in `core/tests/fixtures/mutants_out/` and read
  in stage 1 (open point 9).
- `run_timeout_hours` per `[[repo]]`, defaulting to the roster's, so one large repository can have a longer cap than
  the rest; `deploy/SETUP.md` gains a procedure for measuring a repository and sizing its `jobs` and cap from it
  (open point 3). `OnboardedRepo` carries the cap as `run_timeout`.
- Stage 1, `just stage1`: `cargo fmt`, clippy pedantic with `-D warnings` on every target, the tests. Stage 2,
  `just stage2`: stern4rust pinned at commit `8b0e8e5` of its repository and installed under `target/stern4rust`,
  cargo-shear, dry4rust, crap4rust, twin4rust and iceberg4rust, then the end-to-end tests against the real
  cargo-mutants 27.1.0. Every gate passes with nothing baselined.
- The `xtask` rebuilt as `Gate` implementations against a `CommandRunner`, with tests of its own; a `cargo xtask`
  alias; a `justfile`; the compiler pinned at 1.99.1; CI running both stages on Linux.
- The copyright header on every `.rs` file, named in `stern4rust.toml`.
- `docs/ARCHITECTURE.md`, `docs/ADRs/` (the design's decisions, one per file), `docs/ROADMAP.md`,
  `docs/IMPLEMENTED-FEATURES.md`, `docs/history/`, this changelog, `DONATE.md`, `CODE_OF_CONDUCT.md`, and
  `.github/rulesets/main.json`, the copy of record for `main`'s ruleset.

### Changed
- `Sentinel::new` takes the installed cargo-mutants version, and `RecordCurrency::new` the same beside the
  `config_version`; `CargoMutantsCli::installed_version` reads it.
- The journal has a sixth step, `unreadable`, and `RoundStep` a sixth variant, `Unreadable { reason }`; the
  snapshot's record is a `RecordState` (absent, present, unreadable) instead of an `Option`.
- The library is laid out as the family's: `invocation/` (the command line and `EntryPoint`, the composition root),
  `traits/` (one contract per file), `model/` (with `branch/`, `onboarding/` and `toml/`), `patrol/` (the loop) and
  `system/` (`process/`, `git/`, `mutants/`), one object per file and records where their readers put them. For
  example, `sentinel4rust::git::GitCli` is now `sentinel4rust::system::git::git_cli::GitCli`.
- Behaviour that lived in free functions belongs to named objects: `SummaryToml`, `BranchLayout` and
  `RecordCurrency` for the record, `TomlRoster` for the roster, `PublicationBuilder` and `JournalLine` in the loop,
  `OutcomeInterpreter`, `MutantsOutLists` and `LogTail` for cargo-mutants, `ProcessGroup` and `StreamReader` for
  processes. `GitCli` is a facade over parts with one job each.
- Failures are typed: `checkout` and `publish` fail with `CheckoutError` and `PublishError`, the command line with
  `UsageError`, start-up with `StartupError`, a git command with `GitFailure`, a missing list with `UnreadableList`.
  Each prints what the service printed before.
- `Command` keeps its fields private, with getters and `with_env`; `WorkDir` owns every path under the work
  directory; `Dependencies` holds `Rc<dyn _>`, so one `GitCli` fills two roles; `StdoutJournal` takes the writer it
  prints to; the log-tail sizes are `LogTail::new` arguments chosen in `EntryPoint`.
- Every `#[cfg(test)]` module moved to a mirrored file under `core/tests/`, in Arrange/Act/Assert and named
  `<method>_<conditions>_<result>`, with every assertion; the two tests that read the clock or slept assert on
  captured output and on the clock adapter instead.
- `docs/DESIGN.md` moved, unchanged, to `docs/history/[SUPERSEDED] DESIGN.md`.

### Fixed
- The four statements of the original design that the code contradicted -- publishing, the exit codes' provenance,
  the parser's fixtures, stage 2 having run -- are stated as they hold in ARCHITECTURE and the ADRs, and the stub at
  `docs/DESIGN.md` names them (open point 8).
- Upgrading cargo-mutants no longer needs a `config_version` bump: a record made by another cargo-mutants than the
  installed one is not current, and the commit runs again. The installed version is read once on start; when it
  cannot be read it decides nothing (open point 13).
- A `summary.toml` that does not parse no longer reruns the commit on every visit, silently: the visit logs
  `step=unreadable` with the reason once, backs off to the 6-hour cap and never runs over the record; it recovers
  when the record reads again. `--once` counts it as a failure (open point 4).
- cargo-mutants' copies of the tree no longer go to the service's private `/tmp`, often RAM: each run gets
  `TMPDIR=<work_dir>/tmp/<name>`, removed after the run and, when a killed run left it, on start (open point 1).
- `deploy/SETUP.md` no longer offers `cargo install sentinel4rust`, which installs the 0.0.1 placeholder: it
  installs from the repository with `--path` until 0.1.0 is published (open point 2).
- `deploy/verify-access.sh --include-main` no longer force-pushes `main`: putting it back, and the delete probe,
  carry a lease, so a commit pushed to `main` meanwhile is kept and reported; and it fetches `main`'s whole history,
  so `main` can be put back after the delete probe (open point 14).

### Removed
- The stray `src/main.rs` at the repository root, the 0.0.1 placeholder's leftover.
- The `forward_through_rc!` macro, made unneeded by `Rc<dyn _>`.
