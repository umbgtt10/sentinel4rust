# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-11-13

### The first release

The first version on [crates.io](https://crates.io/crates/sentinel4rust/0.1.0) of both packages: `sentinel4rust`,
the service, in place of the 0.0.1 placeholder that only reserved its name, and `sentinel4rust-core`, its
technology-free core, new. The repository is public from this release on.

The first implementation (`48c8fb1`) is the service as `docs/IMPLEMENTED-FEATURES.md` describes it. Since then the
repository has been brought under the gates and the layout of the other `*4rust` tools. The service's behaviour is
unchanged: the same command line and exit codes, the same roster format, the same journal lines and the same
files and commit messages on `mutants`. The library's paths all changed; it had never been published, and nothing
outside this repository used it.

The repository then splits into a technology-free core and a cli crate
(`docs/history/[IMPLEMENTED] CORE_CLI_SPLIT.md`), with the same behaviour, checked after each step against the tree
before the split.

Then, on 2026-11-13, what a run publishes and reports changes where a run's timeouts went unseen: see Changed.

### Added
- Stage 2 gains a first gate, "Technology-free core": the core's manifest may name no crate, in any section; its
  crate root must carry `#![no_std]` and `#![forbid(unsafe_code)]`; and no file under `core/src` or `core/tests` may
  reach `std::{env,fs,io,net,os,process,thread}`, the printing macros, the file-system methods of `Path` or
  `extern crate std`. It is iceberg4rust's gate, under `xtask/src/gates/boundary/`; the xtask gains `syn`, and
  `tempfile` for its tests.
- A second ruleset per repository, `sentinel: no tags` (`deploy/rulesets/sentinel-no-tags.json`): every tag's
  creation, update and deletion restricted to the repository admin role, since a write key can write every ref and
  the branch ruleset covers no tag. `deploy/onboard.sh` creates it after the branch ruleset, before the deploy key;
  `deploy/verify-access.sh` also tries to create a tag, which must be rejected; `deploy/SETUP.md` section 2 has it by
  hand (open point 20).
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
- `.cargo/mutants.toml`, so sentinel4rust can be onboarded on itself: `sentinel4rust-core` is mutated, against its
  own tests only (`examine_globs = ["core/src/**/*.rs"]`, `test_workspace = false`); `cli/` and `xtask/` are not.
  550 mutants, where the package before the split had 685.
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
  `just stage2`: stern4rust, the release on PATH (0.18.0 or later; a pinned commit installed under
  `target/stern4rust` until that release), cargo-shear, dry4rust, crap4rust, twin4rust and iceberg4rust, then the end-to-end tests against the real
  cargo-mutants 27.1.0. Every gate passes with nothing baselined.
- Stage 1 starts with a lock sync, `just sync`: `cargo update`, every dependency to its newest compatible version,
  in the lock file only; a manifest's requirements never move.
- Stage 2 ends with cargo-deny, judging advisories, licences, bans and sources against `deny.toml`: licences MIT,
  Apache-2.0 and Unicode-3.0, crates.io as the only source, duplicate versions a warning. The advisory database is
  fetched first, up to three times, and the check runs `--frozen`; the exit code, one bit per failed check, is
  decoded into the checks' names. Last, because advisories change without the code changing.
- The `xtask` rebuilt as `Gate` implementations against a `CommandRunner`, with tests of its own; a `cargo xtask`
  alias; a `justfile`; the compiler pinned at 1.99.1; CI running both stages on Linux.
- The copyright header on every `.rs` file, named in `stern4rust.toml`.
- `docs/ARCHITECTURE.md`, `docs/ADRs/` (the design's decisions, one per file), `docs/ROADMAP.md`,
  `docs/IMPLEMENTED-FEATURES.md`, `docs/history/`, this changelog, `DONATE.md`, `CODE_OF_CONDUCT.md`, and
  `.github/rulesets/main.json`, the copy of record for `main`'s ruleset.

### Changed
- A published run whose timeouts outnumber its caught mutants is flagged, by a new `TimeoutWarning`:
  `warning=timeouts-outnumber-caught` on its journal line, before `duration_s`, and on its repository's `--status`
  line, current or behind, which then needs attention -- so `--status`, and `deploy/status-check.sh` with it, exits 1.
  `--help`, the README, `deploy/SETUP.md` and `docs/IMPLEMENTED-FEATURES.md` say so.
- `summary.toml` records `test_timeout_seconds`, the test timeout cargo-mutants set from the baseline, read from
  `mutants.out/debug.log` -- the only file that states it -- by a new `TestTimeoutLine`; absent when the log does not
  say it, and in records written before. A completed `RunOutcome` and the `RunRecord` carry it; a survivor re-check
  keeps the first run's. The `mutants` branch's README names it. Open point 10 and `deploy/SETUP.md` section 4 record
  why it is needed: cargo-mutants 27.1.0 times its baseline without the packages `test_package` names.
- A completed run's commit message on `mutants` names its timeouts when there are any: `Mutants of 1a2b3c4: 0 missed
  (+0 -0), 74 caught, 121 timed out`. Without them, a run whose mutants nearly all timed out -- a test timeout below
  the tests' own run time -- read as a clean one (seen onboarding beggar on the development laptop, 2026-11-13).
- Packages: `sentinel4rust` is now the binary's package (`cli/`), with a library, `sentinel4rust_cli`; the library
  `sentinel4rust` is published by a new package, `sentinel4rust-core` (`core/`). Both share the workspace's version.
  `invocation::*` moves to `sentinel4rust_cli::invocation::*`. The binary is installed with `cargo install --path cli
  --locked`, as `deploy/provision.sh`, `deploy/status-check.sh`, `deploy/SETUP.md` and `deploy/PROVISION.md` now say.
  The core's crate root forbids unsafe code, as the workspace's lints already did.
- Paths are text in the library's model and contracts, which carry them and never treat them as paths: `Command::new`
  takes the directory as `&str` and `Command::dir` returns one; `Roster::work_dir` is a `String`;
  `RepoSource::checkout` returns the checkout as a `String`, and `MutationRunner::run` and `recheck` take it as
  `&str`. Each comes from UTF-8 text -- the command line, the roster, `job.toml` -- and the implementations turn
  it into a `Path` where they touch the disk.
- `MutantsOutLists::read` and `OutcomeInterpreter::interpret` read a run's lists through a new contract,
  `traits::mutants_out::MutantsOut` (`list`, failing with `UnreadableList`), where they took the directory: a
  required list that cannot be read fails as before, an optional one counts as empty as before.
  `system::mutants::mutants_out_dir::MutantsOutDir` reads it on disk, for `CargoMutantsCli`.
- The library keeps what needs no technology, and the implementations move to `sentinel4rust_cli`:
  `system::{git, mutants, process, unit}::*` become `sentinel4rust_cli::system::*`, and `model::work_dir::WorkDir`
  becomes `sentinel4rust_cli::system::work_dir::WorkDir`. Ten files that only read text stay in the library, as
  `protocol::git::{outside_paths, remote_listing, unreachable_classifier}`,
  `protocol::mutants::{log_tail, mutants_out_lists, outcome_interpreter, unreadable_list}` and
  `protocol::unit::{job_error, job_toml, result_toml}`.
- The library is `#![no_std]`, with `alloc`, and still depends on nothing. `Schedule` keeps its states in a
  `BTreeMap`, which it only looks up, so nothing reads them in a different order.
- Stage 2 runs dry4rust once, from the workspace root, with `xtask/` and `core/tests/` excluded by full path, so a
  copy from one crate into another will be seen; what it judges is unchanged. `DryGate::new` takes the exclusions.
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
- The xtask's fakes -- `FakeGate`, `FakeCommandRunner` and `InMemorySourceFiles` -- move to
  `xtask/tests/gates/support/`, one file each. The mirrors that held them, of `Gate`, `CommandRunner` and
  `SourceFiles`, go: those traits carry no behaviour, and twin4rust 0.7.0 finds their mirrors unearned. Ten of the
  fakes' own tests go with them, each already exercised by the gate and stage tests that use the fake; the eleventh,
  that the runner records the program it ran, nothing else checks, and it moves unchanged into `stern_gate_tests.rs`.
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
