# sentinel4rust -- design

- **Status:** Proposed, for the owner's review
- **Date:** 2026-10-23

Sentinels hunt mutants. `sentinel4rust` runs on a dedicated machine, around the clock: it walks a list of
repositories, and whenever a repository's `main` has moved it runs full mutation testing
([cargo-mutants](https://mutants.rs)) against that exact commit, then publishes the surviving mutants, with the
commit hash, on the repository's own `mutants` branch.

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

As a loop:

```
forever:
  for repo in roster:
    fetch repo
    head     = hash of origin/main
    recorded = hash in origin/mutants:summary.toml, if any
    if head == recorded:
      pause 5 min ; continue
    run cargo-mutants on a clean checkout of head
    publish results for head on mutants (orphan branch, created on first use)
    pause 5 min
```

## 2. Decisions

### 2.1 The state lives on the `mutants` branch, not on the machine

"Has `main` changed since the last pull?" is answered by comparing `origin/main` with the hash recorded in
`origin/mutants:summary.toml`. The machine keeps no database: a reboot, a crash or a new disk loses nothing, and
a second machine could take over by cloning. A run that dies half way writes nothing, so the next round simply
runs it again.

### 2.2 `mutants` is an orphan branch

It shares no history with `main` and holds only results, so it can never be merged into `main` by accident, and
its `git log -p` is the history of the survivors -- a mutant appearing in a diff is a test that stopped noticing,
or new code nobody tested.

```
mutants (orphan)
  README.md       what this branch is; generated; "do not merge"
  summary.toml    commit, started/finished UTC, duration, outcome, counts, cargo-mutants and rustc versions
  missed.txt      one surviving mutant per line, sorted
  timeout.txt     one mutant per line that timed out, sorted (absent when none)
```

Lists are sorted so that runs with `-j` > 1, whose order varies, still diff cleanly. Each run overwrites all of
them; git keeps the past. The commit message is `Mutants of <short hash>: <missed> missed, <caught> caught`.

### 2.3 Every run is against an exact commit, in a checkout nobody works in

The run happens in a detached worktree at `head`, never in a working copy with changes, and cargo-mutants'
output goes outside it. The hash published is the hash tested.

### 2.4 It only ever pushes `refs/heads/mutants`

Publishing happens from a second worktree on `mutants`; the push names the ref explicitly. If the push is
rejected (someone else moved `mutants`), the tool refetches, reapplies its files once and retries; failing that,
it logs and moves on -- the next round reruns. Protecting `main` on the remote makes the guarantee hold even
against a bug.

### 2.5 Crate filters live in each repository

Which crates are mutated, and which tests run, is said in the repository's own `.cargo/mutants.toml` on `main`
-- reviewed like code, read by cargo-mutants itself, and the same for a hand run. For example:

```toml
exclude_globs = ["system-tests/**", "xtask/**", "fixture/**"]
test_workspace = true
timeout_multiplier = 3.0
```

The roster carries only what is needed to reach a repository.

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

### 2.7 Time is injected

Every clock read and every pause goes through a trait (stern4rust's `injected-time`), so the loop is tested
without sleeping.

### 2.8 Packaging

A plain binary, `sentinel4rust`, not a cargo subcommand: it is a service pointed at a list, not a command run
inside a project. Both `sentinel4rust` and `cargo-sentinel4rust` are free on crates.io.

## 3. Configuration

```toml
# ~/.config/sentinel4rust/roster.toml
work_dir = "/home/sentinel/sentinel4rust"   # clones, worktrees, cargo-mutants output
pause_seconds = 300
jobs = 6                                     # cargo-mutants -j

[[repo]]
name = "twin4rust"
url = "git@github.com:umbgtt10/twin4rust.git"

[[repo]]
name = "etheram-raft"
url = "git@github.com:umbgtt10/etheram-raft.git"
branch = "main"                              # default
```

Command line: `sentinel4rust --roster <file>` runs forever; `--once` makes one pass over the roster and exits
(manual runs, tests).

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
| `RepoSource` | fetches a repository; reads `main`'s hash and the hash recorded on `mutants` |
| `MutationRunner` | runs cargo-mutants on a checkout at a hash; returns a `RunOutcome` |
| `ResultStore` | publishes a `RunRecord` on `mutants`, creating the orphan branch on first use |

**Data model**

| type | carries |
|---|---|
| `Roster`, `OnboardedRepo` | the list; name, URL, branch |
| `CommitHash` | a validated 40-hex hash |
| `MutantCounts` | caught, missed, timeout, unviable |
| `RunOutcome` | `Completed { counts, missed, timed_out }`, `BaselineFailed { log_tail }`, `ToolFailed { reason }` |
| `RunRecord` | the hash, the times, the versions and the outcome -- what `summary.toml` holds |
| `RoundStep` | what one visit to a repository did: `Unchanged`, `Published(hash)`, `Skipped(reason)` |

**Implementations**

| type | implements |
|---|---|
| `SystemCommandRunner` | `CommandRunner` via `std::process` |
| `SystemClock`, `ThreadPause` | `Clock`, `Pause` |
| `GitCli` | `RepoSource` and `ResultStore`, shelling out to `git` (credentials stay with the system's git and SSH) |
| `CargoMutantsCli` | `MutationRunner`, shelling out to `cargo mutants`, parsing `mutants.out` |
| `TomlRoster` | reads the roster |

**Orchestration:** `Sentinel` visits one repository per step (`visit(&OnboardedRepo) -> RoundStep`) and pauses
after each; `run_forever` and `run_once` loop over the roster.

## 5. Testing

- **Unit, with fakes for every trait:** unchanged `main` pauses and publishes nothing; a moved `main` runs and
  publishes once; the first run creates the branch; a baseline failure publishes `baseline-failed`; a tool
  failure publishes nothing; a rejected push retries once; the pause follows every visit.
- **`GitCli` against real git, no network:** bare repositories in temporary directories with `file://` remotes.
  Asserted: the orphan branch is created with the four files; the recorded hash reads back; `main`'s ref is
  byte-identical after every publish.
- **`CargoMutantsCli`:** the argument list against a fake runner; the parser against `mutants.out` fixtures
  captured from real runs (a completed run with survivors, a baseline failure, a timeout).
- **End to end, in stage 2:** the real cargo-mutants on a three-function fixture crate in a local bare
  repository, `--once`, asserting the published list. It needs cargo-mutants installed, so it is a stage 2 gate,
  not a stage 1 test.
- The family's gates from the first commit: stern4rust (all 35 rules), dry4rust, crap4rust, twin4rust,
  iceberg4rust, cargo-shear.

## 6. The machine

- A user account of its own; `rustup` with each repository's pinned toolchain (its `rust-toolchain.toml` is
  honoured automatically); cargo-mutants installed pinned, `cargo install cargo-mutants --version 27.1.0 --locked`.
- An SSH key with write access to the onboarded repositories -- ideally one deploy key per repository -- and
  `main` protected on the remote.
- A systemd unit (`deploy/sentinel4rust.service`, `Restart=always`), logging to the journal: one line per visit.
- Disk over RAM: each cargo-mutants job builds in its own copy of the tree.

## 7. Implementation slices

Test-first, one commit per slice, both stages green before each.

| # | slice |
|---|---|
| S0 | workspace (`core/`, `xtask/`), header, `stern4rust.toml`, justfile, CLAUDE.md, the gates green on an empty library |
| S1 | data model: `CommitHash`, `MutantCounts`, `RunOutcome`, `RunRecord` (and its `summary.toml` form), `Roster` parsing |
| S2 | contracts, `SystemCommandRunner`, `SystemClock`, `ThreadPause` |
| S3 | `GitCli` read side: fetch, `main`'s hash, the recorded hash -- against bare repositories |
| S4 | `GitCli` publish side: orphan branch, overwrite, commit, push only `mutants`, one retry |
| S5 | `CargoMutantsCli`: arguments, the `mutants.out` parser, the exit code table |
| S6 | `Sentinel`: the visit and the loop, with fakes |
| S7 | command line, `main`, the roster file |
| S8 | the end-to-end gate on a fixture crate |
| S9 | `deploy/` (systemd unit, machine set-up), README, ARCHITECTURE, ADRs for 2.1, 2.2, 2.5 |

## 8. Decisions for the owner

| # | decision | proposed |
|---|---|---|
| 1 | where crate filters live | each repository's `.cargo/mutants.toml` (2.5) |
| 2 | what a red `main` publishes | `baseline-failed` with the log's tail, no list (2.6) |
| 3 | whether the desktop signs its commits on `mutants` | unsigned, from a key used for nothing else |
| 4 | public repositories publish their survivors | yes for the tools; decide for etheram-ibft, whose untested paths sit close to what is kept private |
| 5 | `-j` and `--in-place` | measure on the desktop: `--in-place` keeps `target/` warm but runs one mutant at a time |
| 6 | keep `caught.txt` and `unviable.txt` too, or counts only | counts only |
| 7 | packaging | plain `sentinel4rust` binary (2.8) |
| 8 | a docs-only commit to `main` triggers a full run | yes, as the workflow says; a filter can come later if it costs too much |
