# Architecture

How a `sentinel4rust` process flows through the code, module by module. The decisions behind it are in
[`ADRs/`](ADRs/README.md).

Two crates hold it. `core/` -- package `sentinel4rust-core`, library `sentinel4rust` -- is the logic: `traits/`,
`model/`, `patrol/` and `protocol/`, `#![no_std]`, with no dependency and no technology
([ADR-TechnologyFreeCore](ADRs/ADR-TechnologyFreeCore.md)). `cli/` -- package `sentinel4rust`, library
`sentinel4rust_cli` -- is the binary: `invocation/` and `system/`, every implementation of the core's contracts.
`cli` depends on the core, never the reverse.

## The pipeline

```text
cli/src/main.rs
  └─ EntryPoint::run(args)                     cli/src/invocation/entry_point.rs
      ├─ CommandLine::parse                    --roster <file> [--once | --status], --run-job <dir>, --help,
      │                                        --version; else
      │                                        UsageError, exit 2
      ├─ run_job                               --run-job: JobRunner, inside the run unit
      ├─ status                                --status: discovery once, when the roster names a GitHub App,
      │                                        then StatusReport over its own clones in <work_dir>/status,
      │                                        one StatusLine per repository; exit 1 if one needs attention
      │                                        or discovery failed
      └─ start                                 any StartupError: exit 1
          1. TomlRoster::parse                 the roster file -> Roster
          2. WorkDirLock::acquire              <work_dir>/sentinel4rust.lock, held for the life of the process
          3. JournalLine::startup              the first line on stdout
          4. EntryPoint::compose(roster)       builds every real part, prunes a crash's leftovers, reads
                                               `cargo mutants --version` once -> Sentinel
          5. Sentinel::run_once                --once: each repository once; exit 1 if a visit failed
             Sentinel::run_forever             otherwise: step_at the clock's time, forever

Sentinel::step_at(started)                     core/src/patrol/sentinel.rs; step is step_at the clock's time
  DiscoveryRefresh::refresh(started)           when due -- at start, then every discovery_minutes, after a failure
                                               again after the pause: RepoDiscovery::discover, merged by WatchList
                                               with the roster's own repositories; one `discovery ...` journal line
  Schedule::next(DiscoveryRefresh::repos, now)
    SleepUntil(t) -> Pause::pause(min(t, the next discovery) - now, at least a second)
    Visit(i)      -> visit, then one JournalLine::visit on the Journal (an unreachable or unreadable repeat stays
                     quiet)

Sentinel::visit(repo) -> RoundStep
  1. RepoSource::sync          -> Snapshot { head, record, previous_missed, readiness }: the watched branch
                                  fetched, the results repository's main fetched, the repository's folder read
                                  Unreachable: back off to the cap; Transient: back off; NoBranch: skip
  2. RecordState               unreadable -> Unreadable, backed off to the cap, never run over
     RecordCurrency            present, naming head, config_version and the installed cargo-mutants -> Unchanged
  3. readiness                 not-a-crate / no-filters / path-outside-repo -> Skipped
  4. BaselineRetry             RepoSource::checkout, MutationRunner::run (again once on a baseline failure),
                               MutationRunner::recheck on the new survivors (SurvivorRecheck), RepoSource::remove_checkout
  5. a missing toolchain -> Skipped; a tool failure below the third on this commit -> BackedOff
  6. PublicationBuilder::build -> Publication; ResultStore::publish (one commit replacing the repository's folder in
                                  the results repository) -> Published, or BackedOff if rejected
```

`EntryPoint::compose` is the only place real parts are built, and where a crash's leftovers are pruned. One `SystemCommandRunner` serves git, cargo, curl and openssl; with a GitHub App in the roster, `GithubAccess` wraps it for git in a `GitTokenRunner` and provides the `RepoDiscovery`; one
`GitCli`, given the roster's `results_url`, fills both `RepoSource` and `ResultStore` through `Rc` coercion; `CargoMutantsCli` is the `MutationRunner`;
`SystemClock`, `ThreadPause` and a `StdoutJournal` over stdout complete `Dependencies`, which `Sentinel::new` takes
whole.

## cli: `invocation/` -- the command line and the composition root

`CommandLine` is what the arguments asked for; `UsageError` and `StartupError` say what went wrong, each in the words
the service has always printed. `WorkDirLock` holds an exclusive `File::try_lock` on the work directory's lock file,
so a `--once` beside the running service exits at once. `EntryPoint` turns all of it into an `ExitCode`: 0, 1 for a
start-up failure or a failed `--once` visit, 2 for a bad command line. The usage text and the log-tail sizes (200
lines, 16 KiB) are its constants. It is where the core's text paths -- the roster's `work_dir` -- become `Path`s.
`GithubAccess` turns the roster's GitHub App into its parts, sharing one `AppToken`: the runner git uses
(`GitTokenRunner`) and the discovery (`GithubAppDiscovery` over a `GithubApi`), their files in `sentinel4rust-github/`
beside the App's key ([ADR-GithubAppDiscovery](ADRs/ADR-GithubAppDiscovery.md)).

## core: `traits/` -- the contracts

One trait per file, no default bodies: `CommandRunner` (run a `Command` to completion or its deadline), `Clock`,
`Pause`, `Journal`, `RepoSource` (`sync`, `checkout`, `remove_checkout`, `committed_at`), `MutationRunner` (`run`,
`recheck`), `ResultStore` (`publish`), `RepoDiscovery` (`discover`: the repositories that ask to be watched) and
`MutantsOut` (the text of one list a cargo-mutants run left in `mutants.out`).
The checkout `RepoSource` hands `MutationRunner` is its path, as text. Every external effect goes through one of them,
so the loop is tested with fakes and no clock ([ADR-InjectedTime](ADRs/ADR-InjectedTime.md)).

## core: `model/` -- the data the contracts carry

- **Values:** `CommitHash` (40 hex digits, trimmed, lowercased), `Timestamp` (seconds since the epoch, RFC 3339
  without a time-zone library), `MutantKey` and `Located` (a mutant without and with its position,
  [ADR-PositionFreeMutantKeys](ADRs/ADR-PositionFreeMutantKeys.md)), `KeyChanges` (new and gone), `MutantCounts`.
- **Contract data:** `Command` (its directory as text) and `CommandResult`; `Snapshot` and `SyncError` (what a
  fetch found, or why it did not); `CheckoutError`, `PublishError`; `MutationRun` and `RunOutcome` (what a run
  produced); `OutcomeKind`, `SkipReason` and `RoundStep` (what a visit did, with its journal name).
- **`branch/`** -- what the results repository holds for one repository
  ([ADR-SharedResultsRepository](ADRs/ADR-SharedResultsRepository.md)): the
  `RunRecord` and `Publication` records, `RecordState` (absent, present or unreadable), `SummaryToml` (render and
  parse, schema 1), `RecordCurrency`, `BranchLayout`
  (the root README and the files under the repository's folder, as `BranchFile`s, and the commit message led by the
  folder), `RecordError`, and `KIND_FULL`.
- **`onboarding/`** -- the roster: `Roster` (its `work_dir` and `results_url` as text), `OnboardedRepo` and
  `GithubApp` records, `TomlRoster` (parse and validate), `GithubAppToml` (the three `github_app_*` keys, all or
  none), `RepoNames` (what may name a repository and its branch) and `RosterError`.
- **`github/`** -- what a GitHub App needs, as text ([ADR-GithubAppDiscovery](ADRs/ADR-GithubAppDiscovery.md)):
  `AppJwt` (the RS256 header and claims, short of the signature), `Base64` (the standard alphabet, padded, and the
  URL one, not), the `DiscoveredRepo` record and `DiscoveryError`.
- **`toml/`** -- the dependency-free TOML subset ([ADR-NoDependencies](ADRs/ADR-NoDependencies.md)): `TomlParser`
  builds a `Document` (a root `Table` and arrays of tables) with `DocumentBuilder`, after `CommentStripper` (a
  `QuoteState` machine) has removed comments; `BasicString` reads escapes; `TomlWriter` quotes; `TomlError` names the
  line.

## core: `patrol/` -- the loop

`Sentinel` is the policy: the visit above, the tool-failure count and the publication; `BaselineRetry` runs a commit
in its checkout, once more on a failing baseline, has `SurvivorRecheck` pick the survivors the previous record did not
list and fold a second run on exactly those back in, and always removes the checkout. `Schedule`
keeps one `RepoState` per repository -- last visit, back-off deadline, consecutive failures, tool failures on one
commit, unreachable, unreadable -- and answers `Next`
([ADR-PollingPerRepository](ADRs/ADR-PollingPerRepository.md),
[ADR-ResultsFailuresAndBackOff](ADRs/ADR-ResultsFailuresAndBackOff.md)). `PublicationBuilder` turns a `MutationRun` into
a `Publication`: keys, sorted located lists, new and gone against the previous record. `JournalLine` formats the
start-up line, one `repo=… step=… …` line per visit and one `discovery …` line per discovery. `WatchList` merges the
roster's repositories with the discovered ones -- the roster's first, winning over a discovered one of the same name,
then the rest by name, never the results repository -- and `DiscoveryRefresh` owns the list watched now and when
discovery runs next. `Dependencies` is the record of the six injected parts and the optional `RepoDiscovery`;
`MAX_BACKOFF` (6 hours) is shared from `patrol_constants.rs`. `StatusReport` is the `--status` pass: it syncs
each repository, runs and publishes nothing, and turns what it finds -- and, through `RepoSource::committed_at`, how
long `main`'s head has waited -- into a `StatusLine` that says whether it needs attention. `TimeoutWarning` judges a
run whose timeouts outnumber its caught mutants, for the visit's journal line and for the record's status line.

## core: `protocol/` -- what the tools answer, and what it means

Text in, the core's data out; no tool is run here.

- **`git/`** -- `RemoteListing` (the refs `ls-remote` listed), `UnreachableClassifier` (the failures that mean the
  repository is gone or refuses the key) and `OutsidePaths` (a manifest's `path` dependencies that leave the
  repository, for `path-outside-repo`).
- **`mutants/`** -- `OutcomeInterpreter` (the exit-code table), which reads the lists with `MutantsOutLists`
  through `MutantsOut` (`UnreadableList` when a required one cannot be read) and keeps a failing run's `LogTail`;
  `TestTimeoutLine` reads the test timeout cargo-mutants set out of `debug.log`, the one file that states it.
- **`unit/`** -- `JobToml` and `ResultToml`, the files the service and its run unit hand each other, and `JobError`.

## cli: `system/` -- the real implementations

`WorkDir` owns every path under the roster's `work_dir`: `repos/<name>.git`, `results.git`, `runs/<name>`,
`stage/<name>`, `out/<name>`, `tmp/<name>`, `jobs/`.

- **`process/`** -- `SystemCommandRunner` spawns each command in its own process group with piped output, drains
  stdout and stderr on `StreamReader` threads -- keeping only a tail when the command sets a capture limit, as
  cargo-mutants' runs do (64 KiB) --, polls against the deadline and on expiry has `ProcessGroup` send the
  group `SIGKILL` through `kill`. `SystemClock`, `ThreadPause` and `StdoutJournal` implement the small contracts.
- **`git/`** -- `GitCli` implements `RepoSource` and `ResultStore` over its parts: `GitRunner` (git without prompts,
  in the C locale; `local` failures are `GitFailure`s naming the command), `BareClone` (made once, origin kept on the
  roster's URL, stale worktree entries pruned; `ensure_at` for a clone at a path of its own), `CrateReadiness` (with
  `OutsidePaths`, the manifests' `path` dependencies), `DetachedWorktree` (the run's checkout, with its submodules)
  and `ResultsClone`, with the core's `RemoteListing`, `OutsidePaths` and `UnreachableClassifier`. A watched
  repository's clone fetches its branch and nothing else -- into `refs/remotes/origin/<branch>`, and into
  `refs/heads/<branch>`, where a build in the run unit can fetch it as a private git dependency through `file://`
  (`deploy/SETUP.md` section 2a) -- and nothing is ever pushed to it. `ResultsClone` is the
  results repository's bare clone, `results.git`: `fetch` takes its `main` (or drops the local copy when there is
  none yet), `record` and `previous_missed` read the repository's folder through `RecordReader`, and `publish` has
  `PlumbingCommit` write one commit -- the fetched tip's tree read into a temporary index, the folder's paths removed,
  the new blobs added, `commit-tree` onto the tip -- and pushes it. The deadlines and the results branch's remote ref
  (`RESULTS_REMOTE_REF`) are in `git_constants.rs`, the branch itself in `results_clone.rs`. Only the results repository's `main`
  is ever pushed ([ADR-ExactCommitOnlyMutantsPushed](ADRs/ADR-ExactCommitOnlyMutantsPushed.md),
  [ADR-SharedResultsRepository](ADRs/ADR-SharedResultsRepository.md)).
- **`mutants/`** -- `CargoMutantsCli` re-checks named mutants with an anchored `--re`, and records `cargo mutants --version` and `rustc --version`, runs
  `cargo mutants --jobs <n> --output <out>` in the checkout with the run cap and `TMPDIR` under the work directory
  (its `prune` empties `tmp/` on start), and hands the result to the core's
  `OutcomeInterpreter`, with a `MutantsOutDir` reading the run's `mutants.out` on disk.

- **`github/`** -- the GitHub App, over `openssl` and `curl` through the `CommandRunner`
  ([ADR-GithubAppDiscovery](ADRs/ADR-GithubAppDiscovery.md)): `AppToken` signs the JWT with `openssl dgst -sha256
  -sign`, exchanges it for an installation token through `Curl` and keeps it until five minutes before it expires;
  `GithubApi` GETs with it and checks a URL's status; `GithubAppDiscovery` pages through
  `/installation/repositories` and keeps each not archived with `.cargo/mutants.toml` on its default branch;
  `GitTokenRunner` gives git's network commands the token through the environment; `SecretDir` writes every secret
  mode 600 in a mode-700 directory, so none reaches a command line; `TokenSource` is the seam the API and git share.
  `serde_json` reads the answers.

- **`unit/`** -- when the roster names a `run_unit`, `CargoMutantsCli` runs its commands through
  `UnitCommandRunner`: each is written as `jobs/run/job.toml` (the core's `JobToml`), the template unit started with
  `systemctl start --wait <unit>@run.service`, and the `result.toml` (`ResultToml`), `stdout` and `stderr` the unit
  left are read back into a `CommandResult`. The unit's `ExecStart` is `sentinel4rust --run-job <dir>`: `JobRunner`
  runs the job with the `SystemCommandRunner` and writes those files. The sentinel's own git never runs there, so the
  results repository's key and the App's stay out of the unit ([OPEN_POINTS.md](OPEN_POINTS.md), point 5); the git
  that cargo runs there reads the unit's own configuration, `run-unit.gitconfig`, which sends the owner's GitHub URLs
  to the sentinel's clones.

## Configuration

The roster is TOML, read once on start ([`deploy/roster.example.toml`](../deploy/roster.example.toml)):

| key | default | meaning |
|---|---|---|
| `work_dir` | required | clones, checkouts, staging areas and cargo-mutants output |
| `results_url` | required | the results repository every record is read from and every publication pushed to, one folder per repository |
| `pause_seconds` | 300 | the polling interval per repository |
| `jobs` | 1 | cargo-mutants `--jobs`, for every repository without its own |
| `run_timeout_hours` | 12 | the cap on one cargo-mutants run |
| `config_version` | 1 | bump to rerun every repository |
| `stale_after_hours` | 24 | how long `main` may wait for a record before `--status` exits 1 |
| `run_unit` | none | the systemd template unit cargo-mutants runs in, away from the keys |
| `github_app_id`, `github_app_installation_id`, `github_app_key` | none | the GitHub App that discovers and reads the repositories, its installation, and the path of its private key; all three or none |
| `discovery_minutes` | 60 | how often discovery runs again |
| `[[repo]]` | none with an App; at least one without | a repository watched as given, before and over a discovered one of the same name |
| `[[repo]]` `name`, `url` | required | a unique name (letters, digits, `-`, `_`, `.`), which names its folder in the results repository, and what git fetches from, `https://github.com/<owner>/<repo>.git` |
| `[[repo]]` `branch` | `main` | the branch tested; never `mutants` |
| `[[repo]]` `jobs` | the global `jobs` | this repository's `--jobs` |
| `[[repo]]` `run_timeout_hours` | the global `run_timeout_hours` | this repository's cap on one run |

## Testing

Each crate's `tests/` mirrors its `src/` file by file (`<source>_tests.rs`, one `all_tests.rs` registry), every test
in Arrange/Act/Assert.

`core/tests/` touches no disk and starts no process: its 186 tests run in a hundredth of a second, and they are what
each mutant is tested against. Its `support/` holds `World`, a fake for all six of the sentinel's contracts on a fake
clock; `InMemoryMutantsOut`, the lists of one run held in memory; `run_records` and `command_results`.
`fixtures/mutants_out/` keeps the lists two real cargo-mutants runs wrote, and the end of one's baseline in its
`debug.log`, which the tests of `MutantsOutLists` and `TestTimeoutLine` include at compile time.

`cli/tests/` holds the tests of the real implementations. Its `support/` holds `ScriptedRunner`, a `CommandRunner`
answering from a script, which the git parts are tested against; `Origin`, a bare repository with a working clone, for
`GitCli` and `ResultsClone` on real git through `file://` remotes -- one for the watched repository, one for the
results repository -- with no network; `TempDir`, `git_shell`, `run_records` and a runner
allowing `file://` submodules. `main_tests.rs` drives the binary end to end against a stub `cargo` in stage 1; its
`#[ignore]`d tests run the real cargo-mutants 27.1.0 on three fixture crates -- survivors, a failing baseline, a
mutant that loops -- and stage 2's end-to-end gate runs them.

## Gates

`just stage1`: a lock sync, `cargo fmt`, clippy (pedantic, `-D warnings`, all targets), the tests. `just stage2` is `cargo xtask
stage2`: the technology-free core, stern4rust, cargo-shear, dry4rust, crap4rust, twin4rust, iceberg4rust,
the end-to-end gate, and cargo-deny last. The gates come from serge4rust 0.3.2, which `xtask/src/main.rs` sets with
this repository's constants; the `xtask` keeps two `Gate` implementations of its own against serge4rust's
`CommandRunner`, the end-to-end gate and the iceberg gate, tested with the fake in `xtask/tests/gates/support/`,
`FakeCommandRunner`; CLAUDE.md lists the gates' settings.
