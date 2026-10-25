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
- `deploy/PROVISION.md`: the machine's set-up written for an agent, Claude Code on the machine in the owner's
  checkout. Preconditions first; then numbered steps, each with its command, the check that proves it and when to
  stop and ask; GitHub changed only after the owner's go in the chat, on a plan that lists what changes per
  repository; `gh auth logout` at the end. It covers failures, re-runs, onboarding one more repository, restarting,
  reading the service's state, and a `status-check.sh` roster, cron line and user timer for another machine.
  `deploy/SETUP.md` and the README point to it; `SETUP.md` stays the reference of what each step does by hand, and
  its host aliases gain `StrictHostKeyChecking yes`. Open points 17 to 21 record what only the machine can prove, a
  private repository on the free plan, a restart during a run, tags, and the deploy keys' tie to `gh`.
- `deploy/go-live.sh`, run with sudo: with the service stopped, the first `sentinel4rust --once` pass as
  `sentinel`, in a transient unit with the service's user and limits, so that it outlives the script; its journal
  lines; then, only if no visit failed, `systemctl enable --now sentinel4rust` and a check that it stays active.
  Re-run after an interruption, it waits for the running pass and decides on a quick second one.
  `deploy/status-check.sh`, for another machine: `--status` over a roster with that machine's own access, quiet
  while nothing needs attention, otherwise printing the lines, mailing them with `--mail`, and exiting with
  `--status`'s code, for cron or a systemd user timer.
- `deploy/provision.sh`, run with sudo: `deploy/SETUP.md` sections 1, 3 and 6 as one idempotent script, with
  `--dry-run`, which prints each file it would write or the change to it. It installs the packages, sets up
  unattended security upgrades that reboot at 04:30 when one needs it, and checks the time sync. It creates the
  `sentinel` user, with rustup, Rust 1.99.1 and cargo-mutants 27.1.0, and builds sentinel4rust from a `git archive`
  of the checkout's HEAD (`cargo install --path core --locked`). It writes a roster skeleton that it never
  overwrites, and adds GitHub's SSH host keys, refusing any whose fingerprint GitHub does not publish. It installs
  the two units and the polkit rule (drop-ins for a work directory other than the default), and adds
  `run_unit = "sentinel4rust-mutants"` to the roster only once `deploy/verify-isolation.sh` passes. It does not
  enable the service.
- `deploy/onboard.sh <repo>...`: onboards repositories as `deploy/SETUP.md` section 2 does by hand -- per
  repository the sentinel's key and SSH host alias, on GitHub the ruleset (`deploy/rulesets/sentinel-only-mutants.json`,
  section 2's ruleset as the API body) and then the deploy key with write access, the access proof, and the
  `[[repo]]` entry with the date it was verified. Idempotent, with `--dry-run`, which reads GitHub and writes
  nothing; it changes GitHub only with `--apply-github`, and blocks a repository on which GitHub offers no ruleset.
  `--local <dir>` tests it against `file://` bare repositories, without sudo, GitHub or the access proof.
- `.cargo/mutants.toml`, so sentinel4rust can be onboarded on itself: its core is mutated, `xtask/` left out.
  `deploy/SETUP.md` section 7 says how to decide what a public repository publishes, and the roster example records
  the decision per repository (open point 15).
- `run_unit` in the roster: cargo-mutants then runs in a systemd template unit of its own,
  `deploy/sentinel4rust-mutants@.service`, which cannot read `~/.ssh`, started through the polkit rule in `deploy/`;
  git and the deploy keys stay in the service. The command goes in as a job file and comes back as a result file and
  its output, written by the unit's `sentinel4rust --run-job <dir>`; `deploy/verify-isolation.sh` proves the
  isolation on the machine (open point 5). Not yet proven there.
- `--status`: a read-only pass saying how far each repository's record is behind `main` and how long `main`'s head
  has waited, exiting 1 when one has waited longer than the roster's new `stale_after_hours` (24 by default) or
  could not be judged, so a dead sentinel shows from another machine (open point 12). `RepoSource` gains
  `committed_at`; `Roster` gains `stale_after`; `CommandLine` gains `Status`.
- New survivors -- whose keys the previous record did not list -- are re-checked once in the same checkout with
  `cargo mutants --re` on exactly their names before publishing; one caught on the re-check counts as caught (open
  point 11). `MutationRunner` gains `recheck`; `MutantKey::of_each` keys each line in place.
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
- `BaselineRetry::run` takes the previous record's survivors, to tell the new ones.
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
- Stopping or restarting the service now ends a run in progress in the run unit, which it used to leave running
  for the new service to wait on and read as its own: `deploy/sentinel4rust.service` stops the unit's instances
  before it starts and after it stops, while no other pass holds the work directory's lock (`flock`,
  `SENTINEL4RUST_LOCK`). The polkit rule lets the `sentinel` user stop the run unit as well as start it, and
  `deploy/verify-isolation.sh` proves the stop. The stopped run's repository runs again from the start (open point
  19).
- A cargo-mutants run no longer keeps its whole output in memory for as long as it runs: each stream keeps its last
  64 KiB, `Command::with_capture_limit`; git's output is kept whole. The roster example says the default `jobs` is 1
  (open point 16, which records the rest).
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
