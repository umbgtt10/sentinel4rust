# Architecture

How a `sentinel4rust` process flows through the code, module by module. The decisions behind it are in
[`ADRs/`](ADRs/README.md).

## The pipeline

```text
main.rs
  └─ EntryPoint::run(args)                     invocation/entry_point.rs
      ├─ CommandLine::parse                    --roster <file> [--once], --help, --version; else UsageError, exit 2
      └─ start                                 any StartupError: exit 1
          1. TomlRoster::parse                 the roster file -> Roster
          2. WorkDirLock::acquire              <work_dir>/sentinel4rust.lock, held for the life of the process
          3. JournalLine::startup              the first line on stdout
          4. EntryPoint::compose(roster)       builds every real part, prunes a crash's leftovers, reads
                                               `cargo mutants --version` once -> Sentinel
          5. Sentinel::run_once                --once: each repository once; exit 1 if a visit failed
             Sentinel::run_forever             otherwise: step, forever

Sentinel::step                                 patrol/sentinel.rs
  Schedule::next(repos, now)
    SleepUntil(t) -> Pause::pause(t - now, at least a second)
    Visit(i)      -> visit, then one JournalLine::visit on the Journal (an unreachable or unreadable repeat stays
                     quiet)

Sentinel::visit(repo) -> RoundStep
  1. RepoSource::sync          -> Snapshot { head, record, previous_missed, readiness }
                                  Unreachable: back off to the cap; Transient: back off; NoBranch: skip
  2. RecordState               unreadable -> Unreadable, backed off to the cap, never run over
     RecordCurrency            present, naming head, config_version and the installed cargo-mutants -> Unchanged
  3. readiness                 not-a-crate / no-filters -> Skipped
  4. BaselineRetry             RepoSource::checkout, MutationRunner::run (again once on a baseline failure),
                               RepoSource::remove_checkout
  5. a tool failure below the third on this commit -> BackedOff
  6. PublicationBuilder::build -> Publication; ResultStore::publish -> Published, or BackedOff if rejected
```

`EntryPoint::compose` is the only place real parts are built, and where a crash's leftovers are pruned. One `SystemCommandRunner` serves git and cargo; one
`GitCli` fills both `RepoSource` and `ResultStore` through `Rc` coercion; `CargoMutantsCli` is the `MutationRunner`;
`SystemClock`, `ThreadPause` and a `StdoutJournal` over stdout complete `Dependencies`, which `Sentinel::new` takes
whole.

## `invocation/` -- the command line and the composition root

`CommandLine` is what the arguments asked for; `UsageError` and `StartupError` say what went wrong, each in the words
the service has always printed. `WorkDirLock` holds an exclusive `File::try_lock` on the work directory's lock file,
so a `--once` beside the running service exits at once. `EntryPoint` turns all of it into an `ExitCode`: 0, 1 for a
start-up failure or a failed `--once` visit, 2 for a bad command line. The usage text and the log-tail sizes (200
lines, 16 KiB) are its constants.

## `traits/` -- the contracts

One trait per file, no default bodies: `CommandRunner` (run a `Command` to completion or its deadline), `Clock`,
`Pause`, `Journal`, `RepoSource` (`sync`, `checkout`, `remove_checkout`), `MutationRunner` (`run`) and `ResultStore`
(`publish`). Every external effect goes through one of them, so the loop is tested with fakes and no clock
([ADR-InjectedTime](ADRs/ADR-InjectedTime.md)).

## `model/` -- the data the contracts carry

- **Values:** `CommitHash` (40 hex digits, trimmed, lowercased), `Timestamp` (seconds since the epoch, RFC 3339
  without a time-zone library), `MutantKey` and `Located` (a mutant without and with its position,
  [ADR-PositionFreeMutantKeys](ADRs/ADR-PositionFreeMutantKeys.md)), `KeyChanges` (new and gone), `MutantCounts`.
- **Contract data:** `Command` and `CommandResult`; `Snapshot` and `SyncError` (what a fetch found, or why it did
  not); `CheckoutError`, `PublishError`; `MutationRun` and `RunOutcome` (what a run produced); `OutcomeKind`,
  `SkipReason` and `RoundStep` (what a visit did, with its journal name); `WorkDir`, which owns every path under the
  roster's `work_dir`: `repos/<name>.git`, `runs/<name>`, `stage/<name>`, `out/<name>`, `tmp/<name>`.
- **`branch/`** -- what `mutants` holds ([ADR-OrphanMutantsBranch](ADRs/ADR-OrphanMutantsBranch.md)): the
  `RunRecord` and `Publication` records, `RecordState` (absent, present or unreadable), `SummaryToml` (render and
  parse, schema 1), `RecordCurrency`, `BranchLayout`
  (the files, as `BranchFile`s, and the commit message), `RecordError`, and `KIND_FULL`.
- **`onboarding/`** -- the roster: `Roster` and `OnboardedRepo` records, `TomlRoster` (parse and validate) and
  `RosterError`.
- **`toml/`** -- the dependency-free TOML subset ([ADR-NoDependencies](ADRs/ADR-NoDependencies.md)): `TomlParser`
  builds a `Document` (a root `Table` and arrays of tables) with `DocumentBuilder`, after `CommentStripper` (a
  `QuoteState` machine) has removed comments; `BasicString` reads escapes; `TomlWriter` quotes; `TomlError` names the
  line.

## `patrol/` -- the loop

`Sentinel` is the policy: the visit above, the tool-failure count and the publication; `BaselineRetry` runs a commit
in its checkout, once more on a failing baseline, and always removes the checkout. `Schedule`
keeps one `RepoState` per repository -- last visit, back-off deadline, consecutive failures, tool failures on one
commit, unreachable, unreadable -- and answers `Next`
([ADR-PollingPerRepository](ADRs/ADR-PollingPerRepository.md),
[ADR-ResultsFailuresAndBackOff](ADRs/ADR-ResultsFailuresAndBackOff.md)). `PublicationBuilder` turns a `MutationRun` into
a `Publication`: keys, sorted located lists, new and gone against the previous record. `JournalLine` formats the
start-up line and one `repo=… step=… …` line per visit. `Dependencies` is the record of the six injected parts;
`MAX_BACKOFF` (6 hours) is shared from `patrol_constants.rs`.

## `system/` -- the real implementations

- **`process/`** -- `SystemCommandRunner` spawns each command in its own process group with piped output, drains
  stdout and stderr on `StreamReader` threads, polls against the deadline and on expiry has `ProcessGroup` send the
  group `SIGKILL` through `kill`. `SystemClock`, `ThreadPause` and `StdoutJournal` implement the small contracts.
- **`git/`** -- `GitCli` implements `RepoSource` and `ResultStore` over its parts: `GitRunner` (git without prompts,
  in the C locale; `local` failures are `GitFailure`s naming the command), `BareClone` (made once, origin kept on the
  roster's URL, stale worktree entries pruned), `RemoteListing` (what `ls-remote` listed), `RecordReader` (the
  record and `missed.txt` off the fetched `mutants`), `CrateReadiness`, `DetachedWorktree` (the run's checkout),
  `PlumbingCommit` (blobs, a temporary index and `commit-tree`) and `UnreachableClassifier`. The deadlines and the
  remote `mutants` ref are in `git_constants.rs`. Only `refs/heads/mutants` is ever pushed
  ([ADR-ExactCommitOnlyMutantsPushed](ADRs/ADR-ExactCommitOnlyMutantsPushed.md)).
- **`mutants/`** -- `CargoMutantsCli` records `cargo mutants --version` and `rustc --version`, runs
  `cargo mutants --jobs <n> --output <out>` in the checkout with the run cap and `TMPDIR` under the work directory
  (its `prune` empties `tmp/` on start), and hands the result to
  `OutcomeInterpreter` (the exit-code table), which reads the lists with `MutantsOutLists` (`UnreadableList` when a
  required one is missing) and keeps a failing run's `LogTail`.

## Configuration

The roster is TOML, read once on start ([`deploy/roster.example.toml`](../deploy/roster.example.toml)):

| key | default | meaning |
|---|---|---|
| `work_dir` | required | clones, checkouts, staging areas and cargo-mutants output |
| `pause_seconds` | 300 | the polling interval per repository |
| `jobs` | 1 | cargo-mutants `--jobs`, for every repository without its own |
| `run_timeout_hours` | 12 | the cap on one cargo-mutants run |
| `config_version` | 1 | bump to rerun every repository |
| `[[repo]]` `name`, `url` | required | a unique name (letters, digits, `-`, `_`, `.`) and what git fetches from |
| `[[repo]]` `branch` | `main` | the branch tested; never `mutants` |
| `[[repo]]` `jobs` | the global `jobs` | this repository's `--jobs` |

## Testing

`core/tests/` mirrors `core/src/` file by file (`<source>_tests.rs`, one `all_tests.rs` registry), every test in
Arrange/Act/Assert. Its `support/` holds what several test files share: `World`, a fake for all six of the sentinel's
contracts on a fake clock; `ScriptedRunner`, a `CommandRunner` answering from a script, which the git parts are
tested against; `Origin`, a bare repository with a working clone, for `GitCli` on real git through `file://` remotes,
with no network; `TempDir`, `git_shell` and `run_records`.

`main_tests.rs` drives the binary end to end against a stub `cargo` in stage 1; its one `#[ignore]`d test runs the
real cargo-mutants 27.1.0 on a fixture crate, and stage 2's end-to-end gate runs it.

## Gates

`just stage1`: `cargo fmt`, clippy (pedantic, `-D warnings`, all targets), the tests. `just stage2` is
`cargo xtask stage2`: the pinned stern4rust, cargo-shear, dry4rust, crap4rust, twin4rust, iceberg4rust, then the
end-to-end gate. The `xtask` is a crate of `Gate` implementations against a `CommandRunner`, with tests of its own;
CLAUDE.md lists the gates' settings.
