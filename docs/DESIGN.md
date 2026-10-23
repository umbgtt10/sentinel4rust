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
  for repo in repos:
    if no sentinel4rust.toml naming a crate on its default branch:   (gh, no clone)
      continue                                                       no pause
    fetch repo
    head     = hash of origin/<default branch>
    recorded = hash in origin/mutants:summary.toml, if any
    if head == recorded:
      pause 5 min ; continue
    read sentinel4rust.toml at head
    run cargo-mutants on its crates, on a clean checkout of head
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
a second machine could take over by cloning. A run that dies half way writes nothing, so the next round simply
runs it again. Which repositories are onboarded is not state either: it is read from GitHub and from the
repositories themselves every round.

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

Publishing happens from a second worktree on `mutants`; the push names the ref explicitly. If the push is
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

## 3. Configuration

```toml
# ~/.config/sentinel4rust/config.toml
owners = ["umbgtt10"]                        # GitHub users or organisations whose repositories are listed
work_dir = "/home/sentinel/sentinel4rust"   # clones, worktrees, cargo-mutants output
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
| `MutationRunner` | runs cargo-mutants on given crates of a checkout at a hash; returns a `RunOutcome` |
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
| `RoundStep` | what one look at a repository did: `NotOnboarded`, `Unchanged`, `Published(hash)`, `Skipped(reason)` |

**Implementations**

| type | implements |
|---|---|
| `SystemCommandRunner` | `CommandRunner` via `std::process` |
| `SystemClock`, `ThreadPause` | `Clock`, `Pause` |
| `GhCli` | `RepoDirectory`, shelling out to `gh` (2.9) |
| `GitCli` | `RepoSource` and `ResultStore`, shelling out to `git` (credentials stay with the system's git) |
| `CargoMutantsCli` | `MutationRunner`, shelling out to `cargo mutants --package ...`, parsing `mutants.out`; checks the crate names with `cargo metadata` |
| `TomlSettings` | reads `config.toml` |

**Orchestration:** `Sentinel` lists the repositories once per round, then looks at one per step
(`visit(&DiscoveredRepo) -> RoundStep`), pausing after each visit to an onboarded repository and once after a
round that visited none; `run_forever` and `run_once` drive the rounds.

## 5. Testing

- **Unit, with fakes for every trait:** a repository without the file, or with `crates = []`, is neither cloned
  nor paused for; unchanged `main` pauses and publishes nothing; a moved `main` runs the listed crates and
  publishes once; the crates come from the file at `head`, not from the probe; the first run creates the branch;
  an invalid file or unknown crate publishes `config-invalid`; a baseline failure publishes `baseline-failed`; a
  tool failure publishes nothing; a rejected push retries once; a round with no onboarded repository pauses once.
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
  survivors are published. It needs cargo-mutants installed, so it is a stage 2 gate, not a stage 1 test.
- The family's gates from the first commit: stern4rust (all 35 rules), dry4rust, crap4rust, twin4rust,
  iceberg4rust, cargo-shear.

## 6. The machine

- A user account of its own; `rustup` with each repository's pinned toolchain (its `rust-toolchain.toml` is
  honoured automatically); cargo-mutants installed pinned, `cargo install cargo-mutants --version 27.1.0 --locked`.
- `gh`, logged in (`gh auth login`) with a credential that can list and read every repository of the owners,
  private ones included.
- Write access for pushing `mutants` (decision 3 below), and `main` protected on the remote.
- A systemd unit (`deploy/sentinel4rust.service`, `Restart=always`), logging to the journal: one line per visit,
  and one per round with the number of repositories listed and onboarded.
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
| S6 | `CargoMutantsCli`: arguments, the crate check, the `mutants.out` parser, the exit code table |
| S7 | `Sentinel`: discovery, the visit and the loop, with fakes |
| S8 | command line, `main`, the config file |
| S9 | the end-to-end gate on a fixture workspace |
| S10 | `deploy/` (systemd unit, machine set-up), README, ARCHITECTURE, ADRs for 2.1, 2.2, 2.5, 2.9 |

## 8. Decisions for the owner

Decided on 2026-10-23:

- Repositories are discovered, not listed by hand; a repository opts in with `/sentinel4rust.toml` (2.5).
- The file names the crates to mutate; absent or empty means skipped (2.5).
- `.cargo/mutants.toml` keeps the run settings (2.5).
- Discovery goes through the `gh` CLI (2.9).

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
