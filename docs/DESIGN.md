# sentinel4rust -- design

- **Status:** Proposed, for the owner's review
- **Date:** 2026-10-23

Sentinels hunt mutants. `sentinel4rust` runs on a dedicated machine, around the clock: it lists every repository
an owner has on GitHub, and in each one that opts in -- with a `sentinel4rust.toml` at its root naming the crates
to check -- whenever the default branch has moved it runs full mutation testing
([cargo-mutants](https://mutants.rs)) on those crates at that exact commit, then publishes the surviving mutants,
with the commit hash, on the repository's own `mutants` branch.

It is not a gate. Nothing waits for it, and it never touches `main`. It answers one question per commit, after
the fact: *which changes to this code would its tests not notice?*

---

## 1. The workflow

The owner's workflow, as given on 2026-10-23:

1. pull the next repository from an onboarded list;
2. if `main` has changed since the last pull, go ahead; otherwise wait 5 minutes and move to the next repository;
3. create the branch `mutants` if it does not exist;
4. run full mutation testing against `main`, noting its commit hash (some crates filtered out);
5. overwrite the mutants list on `mutants`, with the commit hash;
6. commit and push it; wait 5 minutes; move to the next repository.

Steps 5 and 6 are read as one: the list is written, committed and pushed, and only then does the pause start.

Amended the same day: the onboarded list is not kept by hand. Each round the tool lists all of the owner's
repositories, and a repository is onboarded exactly when its default branch carries a `sentinel4rust.toml` at the
root naming at least one crate (2.5). That file is also what filters the crates.

As a loop:

```
forever:
  repos = every non-fork, non-archived repository of each owner      (gh)
  sort repos by owner/name                                           (2.11)
  move first any repository with an interrupted run on disk          (2.10)
  for repo in repos:
    if no sentinel4rust.toml naming a crate on its default branch:   (gh, no clone)
      continue                                                       no pause
    fetch repo
    head     = hash of origin/<default branch>
    recorded = hash in origin/mutants:summary.toml, if any
    if head == recorded:
      pause 5 min ; continue
    read sentinel4rust.toml at head
    run cargo-mutants on its crates, on a clean checkout of head,
      resuming an interrupted run at head if there is one             (2.10)
    publish results for head on mutants (orphan branch, created on first use)
    pause 5 min
  if the round visited no onboarded repository:
    pause 5 min
```

Repositories that have not opted in cost one listing entry and one file probe per round: no clone, no pause. The
pause still follows every visit to an onboarded repository, unchanged or not, as the workflow says.

## 2. Decisions

### 2.1 The state lives on the `mutants` branch, not on the machine

"Has `main` changed since the last pull?" is answered by comparing `origin/main` with the hash recorded in
`origin/mutants:summary.toml`. The machine keeps no database: a reboot, a crash or a new disk loses nothing, and
a second machine could take over by cloning. A run that dies half way publishes nothing, so the next round runs
it again -- resuming from what it had already caught (2.10). Which repositories are onboarded is not state
either: it is read from GitHub and from the repositories themselves every round.

Everything under `work_dir` -- clones, worktrees, cargo-mutants output, run ledgers -- is a cache: deleting it
costs time, never correctness.

### 2.2 `mutants` is an orphan branch

It shares no history with `main` and holds only results, so it can never be merged into `main` by accident, and
its `git log -p` is the history of the survivors -- a mutant appearing in a diff is a test that stopped noticing,
or new code nobody tested.

```
mutants (orphan)
  README.md       what this branch is; generated; "do not merge"
  summary.toml    commit, crates, started/finished UTC, duration, outcome, counts, cargo-mutants and rustc versions
  missed.txt      one surviving mutant per line, sorted
  timeout.txt     one mutant per line that timed out, sorted (absent when none)
```

Lists are sorted so that runs with `-j` > 1, whose order varies, still diff cleanly. Each run overwrites all of
them; git keeps the past. The commit message is `Mutants of <short hash>: <missed> missed, <caught> caught`.

### 2.3 Every run is against an exact commit, in a checkout nobody works in

The run happens in a detached worktree at `head`, never in a working copy with changes, and cargo-mutants'
output goes outside it. The hash published is the hash tested -- and, because the crate list is read from that
same commit, the hash also says which crates were tested.

### 2.4 It only ever pushes `refs/heads/mutants`

Publishing happens from a second worktree on `mutants`, reset to `origin/mutants` before anything is written, so
a commit left unpushed by a crash is dropped rather than built on; the push names the ref explicitly. If the push is
rejected (someone else moved `mutants`), the tool refetches, reapplies its files once and retries; failing that,
it logs and moves on -- the next round reruns. Protecting `main` on the remote makes the guarantee hold even
against a bug.

### 2.5 Each repository opts in, and names its crates, with `/sentinel4rust.toml`

A repository is onboarded by committing one file at the root of its default branch:

```toml
# sentinel4rust.toml
crates = ["twin4rust", "twin4rust-core"]
```

- `crates` is required: the package names, as in each crate's `[package] name`, that cargo-mutants mutates. They
  are passed as `cargo mutants --package <name>`, once per crate.
- The file is absent, or `crates = []`: the repository is skipped -- no clone, no run, nothing published. An empty
  list is the explicit "onboarded, nothing to check right now"; whatever `mutants` already holds stays, with the
  hash it was for.
- The file is parsed strictly: unknown keys, duplicates, a missing `crates` or malformed TOML are errors, so a
  typo such as `crate = [...]` fails loudly instead of quietly opting out. So is a name that is not a package of
  the workspace at `head`, checked with `cargo metadata --no-deps`. An invalid file publishes `config-invalid`
  with the reason (2.6), recorded against the hash like any outcome, so it is not retried until `main` moves.
- The file is read at the commit being tested, so changing what is checked is an ordinary, reviewed commit to
  `main`.

How the listed crates are mutated -- excluded files, which tests run, timeouts -- stays in the repository's own
`.cargo/mutants.toml`, read by cargo-mutants itself and the same for a hand run. For example:

```toml
exclude_globs = ["fixture/**"]
test_workspace = true
timeout_multiplier = 3.0
```

So the two files split cleanly: `sentinel4rust.toml` says *whether* and *which crates*; `.cargo/mutants.toml`
says *how*.

### 2.6 How results are read from cargo-mutants

cargo-mutants 27.1 exits `0` (all caught), `2` (some missed), `3` (some timed out), `4` (baseline failed: the
unmutated tree's own build or tests failed), `1`/`5`/`6`/`70` (usage, diff filter, internal error). `3` takes
precedence over `2`, so the tool reads `mutants.out/outcomes.json` and the `.txt` lists rather than the exit code
alone.

| exit | outcome published | lists |
|---|---|---|
| 0, 2, 3 | `completed` | `missed.txt`, `timeout.txt` |
| 4 | `baseline-failed`, with the log's tail | none: a red `main` has no survivors to report |
| 1, 5, 6, 70, or the process could not start | nothing; logged; retried next round | -- |
| not run: `sentinel4rust.toml` invalid at `head` | `config-invalid`, with the reason | none |

### 2.7 Time is injected

Every clock read and every pause goes through a trait (stern4rust's `injected-time`), so the loop is tested
without sleeping.

### 2.8 Packaging

A plain binary, `sentinel4rust`, not a cargo subcommand: it is a service pointed at an owner, not a command run
inside a project. Both `sentinel4rust` and `cargo-sentinel4rust` are free on crates.io.

### 2.9 Repositories are discovered through `gh`

The listing is `gh repo list <owner> --source --no-archived --limit 1000 --json
nameWithOwner,url,sshUrl,defaultBranchRef,isEmpty`: forks, archived and empty repositories are left out, and the
default branch is taken from GitHub rather than assumed to be `main`. The probe for the file, before any clone, is
`gh api repos/<owner>/<name>/contents/sentinel4rust.toml?ref=<default branch>` with the raw media type; a 404
means not onboarded. Credentials stay with the system's `gh`, as git's stay with git: the crate holds no token and
needs no HTTP client. A listing that comes back with exactly the limit is logged as possibly truncated.

### 2.10 An interrupted run resumes

A full run can take hours, so a crash, a reboot or a restart of the service should not throw away what it had
already caught. cargo-mutants offers the means: it writes `caught.txt` and its sibling lists one line per mutant
as each finishes -- they survive a `SIGKILL` -- and `--iterate` skips every mutant listed in the previous
`mutants.out/caught.txt` and `unviable.txt`. Tried on 2026-10-23 with cargo-mutants 27.1.0: a 28-mutant crate
killed after 17 caught, resumed with `--iterate`, tested only the other 11, and the totals matched a fresh full
run; a seed directory holding nothing but a hand-written `caught.txt` was honoured as well.

Two things the experiment also showed, and which the sentinel must make up for:

- cargo-mutants keeps one previous run only (`mutants.out.old`), and a resumed run's `caught.txt` lists only
  what *it* caught. A second crash would lose the first attempt's progress.
- `--iterate` does not check what it skips against: it would skip mutants caught at another commit as readily.

So the sentinel keeps its own ledger, one per interrupted run:

```
work_dir/<owner>/<name>/ledger/<hash>/
  run.toml          commit, crates, cargo-mutants and rustc versions, attempts so far
  caught.txt        union of every attempt's caught mutants, sorted
  unviable.txt      union of every attempt's unviable mutants, sorted
  attempt-<n>/      the -o directory of attempt n
```

- **Before an attempt**, whatever the previous attempt's lists hold -- complete or cut short -- is merged into the
  ledger's unions; then `attempt-<n>/mutants.out/` is seeded with the unions alone and cargo-mutants runs with
  `-o attempt-<n> --iterate`.
- **The ledger is used only if it matches**: `run.toml`'s commit equals `head`, its crates equal the file's at
  `head`, and the installed cargo-mutants and rustc versions are the ones recorded. `.cargo/mutants.toml` and
  `rust-toolchain.toml` are covered by the hash. Any mismatch -- `main` moved, a toolchain upgraded -- deletes the
  ledger and starts afresh: the sentinel tests the newest `main`, never an old one to finish it.
- **Missed and timed-out mutants are tested again** in every attempt, so the published `missed.txt` and
  `timeout.txt` are the last attempt's; the caught and unviable counts are the unions'. Each attempt also reruns
  the baseline -- one clean build and test, the price of resuming.
- **Interrupted runs come first**: a round moves any repository with a ledger to the front, so a restart finishes
  the cut-short run instead of walking the whole list, and its pauses, to get back to it.
- **The ledger is deleted** once its outcome is published (`completed`, `baseline-failed`), and when its
  repository is no longer onboarded. A `ToolFailed` attempt keeps it, so the retry resumes too.

The ledger lives on the machine and the state does not: losing a ledger costs the hours it saved and nothing
else (2.1).

### 2.11 Repositories take turns, in a fixed order

Decided on 2026-10-23: round-robin. Each round lists the repositories afresh, sorts them by `owner/name`, moves
any with a ledger to the front (2.10), and visits each onboarded repository once, in that order.

- **No starvation.** A repository gets at most one run per round, so however busy the others are, a commit waits
  at most one round -- the sum of that round's runs and pauses -- before it is tested.
- **The newest commit wins.** `head` is read when the repository's turn comes, not when the round starts, so
  several pushes during someone else's three-hour run cost one run, of the latest.
- **Predictable.** The order depends on names only, not on GitHub's sorting or on push times, so the journal of
  one round reads like the next, and tests can pin it.
- **New repositories join at the next round.** The listing is taken once, when a round starts; the probe for
  `sentinel4rust.toml` happens at each repository's turn. So a repository created during a round is first seen
  at the next one, in its alphabetical place; an existing repository that gains the file is picked up this round
  if its turn is still to come, otherwise at the next. The wait is at most the rest of the current round, and no
  run is interrupted for a newcomer. Re-listing before every visit, or letting never-scanned repositories jump
  the queue, were considered on 2026-10-23 and set aside: onboarding is rare, and the next round is soon enough.
- **Leaving is just as quiet.** A repository deleted or archived mid-round probes as absent at its turn and is
  skipped; one that removes the file or empties `crates` is skipped and its ledger deleted (2.10).
- **The cost**: the pause after every visit, as the workflow says, means a quiet round of N onboarded
  repositories takes N x `pause_seconds`. Longest-waiting-first and most-recently-pushed-first were considered
  and set aside for this simplicity; either could replace the sort later without touching the rest.

## 3. Configuration

```toml
# ~/.config/sentinel4rust/config.toml
owners = ["umbgtt10"]                        # GitHub users or organisations whose repositories are listed
work_dir = "/home/sentinel/sentinel4rust"   # clones, worktrees, cargo-mutants output, ledgers: a cache
pause_seconds = 300
jobs = 6                                     # cargo-mutants -j
```

No repository is named here: the list is discovered (2.9) and each repository opts itself in (2.5).

Command line: `sentinel4rust --config <file>` runs forever; `--once` makes one pass over the discovered
repositories and exits (manual runs, tests).

## 4. Architecture

A workspace like the other tools: `core/` (package `sentinel4rust`, library and binary) and `xtask/` (the gates).
Contracts first, then the data model, then the implementations; every dependency injected through the single
constructor.

**Contracts (`traits/`)**

| trait | does |
|---|---|
| `CommandRunner` | runs a program with arguments in a directory, returns exit code, stdout, stderr |
| `Clock` | the current UTC time |
| `Pause` | waits for a duration |
| `RepoDirectory` | lists an owner's repositories; reads one file from a repository's default branch without cloning |
| `RepoSource` | fetches a repository; reads the default branch's hash, the hash recorded on `mutants`, and a file at a hash |
| `MutationRunner` | runs cargo-mutants on given crates of a checkout at a hash, seeded with mutants already caught; returns a `RunOutcome` |
| `LedgerStore` | finds, creates, merges into and deletes the ledger of an interrupted run (2.10) |
| `ResultStore` | publishes a `RunRecord` on `mutants`, creating the orphan branch on first use |

**Data model**

| type | carries |
|---|---|
| `Settings` | owners, work directory, pause, jobs -- what `config.toml` holds |
| `DiscoveredRepo` | owner and name, clone URL, default branch |
| `SentinelFile`, `CrateName` | the parsed `sentinel4rust.toml`: a list of distinct crate names, possibly empty |
| `CommitHash` | a validated 40-hex hash |
| `MutantCounts` | caught, missed, timeout, unviable |
| `RunOutcome` | `Completed { counts, missed, timed_out }`, `BaselineFailed { log_tail }`, `ConfigInvalid { reason }`, `ToolFailed { reason }` |
| `RunRecord` | the hash, the crates, the times, the versions and the outcome -- what `summary.toml` holds |
| `Ledger` | an interrupted run: hash, crates, versions, attempts, and the caught and unviable unions -- what `run.toml` and its lists hold |
| `RoundStep` | what one look at a repository did: `NotOnboarded`, `Unchanged`, `Published(hash)`, `Skipped(reason)` |

**Implementations**

| type | implements |
|---|---|
| `SystemCommandRunner` | `CommandRunner` via `std::process` |
| `SystemClock`, `ThreadPause` | `Clock`, `Pause` |
| `GhCli` | `RepoDirectory`, shelling out to `gh` (2.9) |
| `GitCli` | `RepoSource` and `ResultStore`, shelling out to `git` (credentials stay with the system's git) |
| `CargoMutantsCli` | `MutationRunner`, shelling out to `cargo mutants --package ... --iterate`, seeding and parsing `mutants.out`; checks the crate names with `cargo metadata` |
| `FsLedgerStore` | `LedgerStore`, under `work_dir` |
| `TomlSettings` | reads `config.toml` |

**Orchestration:** `Sentinel` lists the repositories once per round, moves those with a ledger to the front,
then looks at one per step
(`visit(&DiscoveredRepo) -> RoundStep`), pausing after each visit to an onboarded repository and once after a
round that visited none; `run_forever` and `run_once` drive the rounds.

## 5. Testing

- **Unit, with fakes for every trait:** a repository without the file, or with `crates = []`, is neither cloned
  nor paused for; unchanged `main` pauses and publishes nothing; a moved `main` runs the listed crates and
  publishes once; the crates come from the file at `head`, not from the probe; the first run creates the branch;
  an invalid file or unknown crate publishes `config-invalid`; a baseline failure publishes `baseline-failed`; a
  tool failure publishes nothing; a rejected push retries once; a round with no onboarded repository pauses once.
- **Scheduling, with fakes:** repositories are visited sorted by `owner/name` whatever order the listing returns;
  each onboarded repository is visited once per round; a repository that moves twice during another's run is run
  once, at its latest head.
- **Resuming, with fakes:** a repository with a ledger is visited first; a matching ledger seeds the run; a ledger
  for another hash, other crates or other versions is deleted and the run starts afresh; a publish deletes the
  ledger, a tool failure keeps it; the published caught count is the union, the missed list the last attempt's.
- **`FsLedgerStore`, in temporary directories:** merging a cut-short attempt's lists, including a last line
  without its newline; two interrupted attempts in a row lose nothing; `run.toml` round-trips.
- **`SentinelFile`:** the strict parse -- unknown keys, duplicates, missing `crates`, malformed TOML -- each
  rejected with a message naming the problem.
- **`GhCli`:** the argument lists against a fake runner; the parser against `gh repo list` JSON captured from a
  real run; a 404 from `gh api` read as "absent", any other failure as an error. No network.
- **`GitCli` against real git, no network:** bare repositories in temporary directories with `file://` remotes.
  Asserted: the orphan branch is created with the four files; the recorded hash reads back; a file reads back at
  a given hash; `main`'s ref is byte-identical after every publish.
- **`CargoMutantsCli`:** the argument list, one `--package` per crate, against a fake runner; the parser against
  `mutants.out` fixtures captured from real runs (a completed run with survivors, a baseline failure, a timeout).
- **End to end, in stage 2:** the real cargo-mutants on a two-crate fixture workspace in a local bare repository,
  with `sentinel4rust.toml` naming one crate and a fake `RepoDirectory`, `--once`, asserting that only that crate's
  survivors are published. A second run starts from a ledger seeded with half of the caught mutants and asserts
  that only the rest are tested and that the published result equals the first. It needs cargo-mutants
  installed, so it is a stage 2 gate, not a stage 1 test.
- The family's gates from the first commit: stern4rust (all 35 rules), dry4rust, crap4rust, twin4rust,
  iceberg4rust, cargo-shear.

## 6. The machine

- A user account of its own; `rustup` with each repository's pinned toolchain (its `rust-toolchain.toml` is
  honoured automatically); cargo-mutants installed pinned, `cargo install cargo-mutants --version 27.1.0 --locked`.
- `gh`, logged in (`gh auth login`) with a credential that can list and read every repository of the owners,
  private ones included.
- Write access for pushing `mutants` (decision 3 below), and `main` protected on the remote.
- A systemd unit (`deploy/sentinel4rust.service`), enabled at boot, `Restart=always`, logging to the journal: one
  line per visit, and one per round with the number of repositories listed and onboarded. Its default
  `KillMode=control-group` takes cargo-mutants and its test processes down with the sentinel, so a restart never
  finds an orphaned run still holding the tree; the ledger (2.10) picks up where it stopped.
- Disk over RAM: each cargo-mutants job builds in its own copy of the tree.

## 7. Implementation slices

Test-first, one commit per slice, both stages green before each.

| # | slice |
|---|---|
| S0 | workspace (`core/`, `xtask/`), header, `stern4rust.toml`, justfile, CLAUDE.md, the gates green on an empty library |
| S1 | data model: `CommitHash`, `MutantCounts`, `RunOutcome`, `RunRecord` (and its `summary.toml` form), `SentinelFile` and `Settings` parsing |
| S2 | contracts, `SystemCommandRunner`, `SystemClock`, `ThreadPause` |
| S3 | `GhCli`: the listing and the file probe, against captured output |
| S4 | `GitCli` read side: fetch, the default branch's hash, the recorded hash, a file at a hash -- against bare repositories |
| S5 | `GitCli` publish side: orphan branch, overwrite, commit, push only `mutants`, one retry |
| S6 | `CargoMutantsCli`: arguments, the crate check, seeding, the `mutants.out` parser, the exit code table |
| S7 | `Ledger` and `FsLedgerStore`: create, match, merge, delete |
| S8 | `Sentinel`: discovery, the visit, resuming and the loop, with fakes |
| S9 | command line, `main`, the config file |
| S10 | the end-to-end gate on a fixture workspace, fresh and resumed |
| S11 | `deploy/` (systemd unit, machine set-up), README, ARCHITECTURE, ADRs for 2.1, 2.2, 2.5, 2.9, 2.10, 2.11 |

## 8. Decisions for the owner

Decided on 2026-10-23:

- Repositories are discovered, not listed by hand; a repository opts in with `/sentinel4rust.toml` (2.5).
- The file names the crates to mutate; absent or empty means skipped (2.5).
- `.cargo/mutants.toml` keeps the run settings (2.5).
- Discovery goes through the `gh` CLI (2.9).
- An interrupted run resumes from a ledger on the machine, which stays a cache (2.10).
- Repositories take turns round-robin, sorted by `owner/name`, interrupted runs first (2.11).

Open:

| # | decision | proposed |
|---|---|---|
| 1 | which owners are listed | `umbgtt10` only; organisations can be added to `owners` later |
| 2 | what a red `main` publishes | `baseline-failed` with the log's tail, no list (2.6) |
| 3 | how the machine pushes `mutants` | one fine-grained token for all of the owner's repositories (Contents: read and write), used by git through `gh auth setup-git` over HTTPS: with discovery, per-repository deploy keys would make onboarding two steps -- the file and a key |
| 4 | whether the desktop signs its commits on `mutants` | unsigned |
| 5 | public repositories publish their survivors | yes for the tools; decide for etheram-ibft, whose untested paths sit close to what is kept private |
| 6 | `-j` and `--in-place` | measure on the desktop: `--in-place` keeps `target/` warm but runs one mutant at a time |
| 7 | keep `caught.txt` and `unviable.txt` too, or counts only | counts only |
| 8 | packaging | plain `sentinel4rust` binary (2.8) |
| 9 | a docs-only commit to `main` triggers a full run | yes, as the workflow says; a filter can come later if it costs too much |
