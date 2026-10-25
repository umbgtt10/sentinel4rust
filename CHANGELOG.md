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

### Removed
- The stray `src/main.rs` at the repository root, the 0.0.1 placeholder's leftover.
- The `forward_through_rc!` macro, made unneeded by `Rc<dyn _>`.
