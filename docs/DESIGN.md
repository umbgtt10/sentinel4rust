# sentinel4rust -- design

- **Status:** Implemented (revision 3.1: the notes in section 9 record where the code differs from revision 3)
- **Date:** 2026-10-23

Sentinels hunt mutants. `sentinel4rust` runs on a dedicated machine, around the clock: it walks a list of
repositories, and whenever a repository's `main` has moved it runs full mutation testing
([cargo-mutants](https://mutants.rs)) against that exact commit, then publishes the surviving mutants, with the
commit hash, on the repository's own `mutants` branch.

It is not a gate. Nothing waits for it, and it never touches `main`. It answers one question per commit, after
the fact: *which changes to this code would its tests not notice?*

It is a personal tool: one user, one machine under the desk, nothing else running on it. The design stays as
small as that allows.

---

## 1. The loop

1. take the next repository from the roster;
2. if its `main` has not moved since the last published result, move on;
3. otherwise run full mutation testing against `main`'s exact commit (crates filtered by the repository's own
   config);
4. overwrite the results on the `mutants` branch (created on first use), with the commit hash; commit and push.

The 5-minute wait is a **polling interval per repository**: no repository is visited twice within
`pause_seconds`, and the machine sleeps only when none is due. Among due repositories, the one that has waited
longest goes first (ties in roster order), so a busy repository cannot starve the rest. With one repository this is a plain "check every
5 minutes"; with twelve, each is still checked every 5 minutes instead of once an hour.

```
forever:
  repo = next due repository in roster order     # due: last visit >= pause ago, and not backing off
  if none: sleep until the earliest is due ; continue
  fetch repo
  head     = hash of origin/main
  recorded = record in origin/mutants:summary.toml, if any
  if recorded is current for head: mark visited ; continue
  run cargo-mutants on a detached worktree at head
  completed or baseline-failed -> publish on mutants
  tool failure                 -> back off
  mark visited
```

One repository runs at a time. A long run delays the others; the run cap (2.6) bounds the delay. If `main` moves
during a run, the run publishes the hash it tested and the next visit picks up the new one.

## 2. Decisions

### 2.1 State lives on `mutants`

"Has `main` moved?" compares `origin/main` with the commit in `origin/mutants:summary.toml`. A record is
**current** when its commit equals `head` and its `config_version` equals the roster's; bumping `config_version`
(for instance after upgrading cargo-mutants) reruns everything once. The machine keeps no database: a reboot loses
only the in-memory schedule and back-off, which costs at most an early visit. A run that dies half way publishes
nothing and is simply rerun.

The fetch uses explicit refspecs for `main` and, if `git ls-remote --heads origin mutants` shows it exists,
`mutants`; a missing `mutants` means "no record". Clones are full, never shallow. The roster's `branch` (default
`main`) is used wherever this document says `main`.

### 2.2 `mutants` is an orphan branch

It shares no history with `main`, so it cannot be merged by accident, and its `git log -p` is the history of the
survivors.

```
mutants (orphan)
  README.md       what this branch is; generated; "do not merge"
  summary.toml    schema = 1, commit, config_version, kind = "full", started/finished UTC, duration,
                  outcome, counts (caught, missed, timeout, unviable), new/gone, cargo-mutants and rustc
                  versions, effective arguments, log tail (failures only)
  missed.txt      one surviving mutant per line, by key (2.3), sorted
  timeout.txt     one timed-out mutant per line, by key, sorted (absent when none)
  located/        the same lists as cargo-mutants wrote them, with file:line:col
```

Each run overwrites everything; git keeps the past. Commit message:
`Mutants of <short hash>: <missed> missed (+<new> -<gone>), <caught> caught`, or
`Mutants of <short hash>: baseline failed` / `tool failed`.

### 2.3 Mutants are identified without line numbers

cargo-mutants names mutants with `file:line:col`, so one line added at the top of a file would make the whole list
look removed and re-added. The published lists use a **key**: the line as cargo-mutants wrote it with `:line:col`
removed (`src/lib.rs: replace == with != in Parser::next`), with an ordinal (`#2`) in source order when the same
key occurs twice in a file. New and gone are the set differences with the previous record. Timeouts never count
as new. *(Keys come from the `.txt` lists, not `outcomes.json`: see 9.)*

### 2.4 Runs are against an exact commit; only `mutants` is pushed

Each run uses a detached worktree at `head`, nobody's working copy; cargo-mutants' output goes outside it; both are
removed afterwards, whatever the outcome, and leftovers are pruned on start. Publishing uses a second worktree on
`mutants` and `git push origin HEAD:refs/heads/mutants`. A rejected push is logged and the next visit reruns. The
rulesets (5.1) make "only `mutants`" hold even against a bug.

### 2.5 Crate filters, and accepted survivors, live in each repository

Which crates are mutated, which tests run, and which survivors are accepted as equivalent mutants are all said in
the repository's `.cargo/mutants.toml` (`exclude_globs`, `exclude_re`, `test_workspace`, `timeout_multiplier`) or
with `#[mutants::skip]` -- reviewed like code, and the same for a hand run. Nothing is configured per repository on
the `mutants` branch. A repository without `.cargo/mutants.toml` is skipped (`no-filters`) so nothing is mutated
that was meant to be excluded.

### 2.6 Reading results, failures and timeouts

cargo-mutants 27.1 exits `0` (all caught), `2` (missed), `3` (timeouts), `4` (baseline failed), `1`/`5`/`6`/`70`
(usage, diff filter, internal error); the tool reads `mutants.out` rather than the exit code alone. *(Checked
against 27.1.0 in S5.)*

| result | handling |
|---|---|
| exit 0, 2, 3 | publish `completed` with the lists |
| exit 4 | rerun once at once (flaky tests); a second 4 publishes `baseline-failed` with the log tail, no lists |
| exit 1, 5, 6, 70, cannot start, run cap hit | publish nothing; back off; the 3rd failure on the same commit publishes `tool-failed` with the log tail |

`baseline-failed` and `tool-failed` record the commit, so they wait for `main` to move. Log tails are capped at
200 lines / 16 KiB; nothing is redacted (5.1).

**Back-off:** after a failure the next attempt waits `pause_seconds × 2^(failures-1)`, capped at 6 hours; any
success, unchanged `main` or new commit resets it.

**Timeouts:** every command runs with a deadline and is killed with its process group on expiry -- `git fetch`,
`ls-remote` and `push` 5 minutes, other git 1 minute, the whole cargo-mutants run `run_timeout_hours` (12). A
network failure backs off like a tool failure.

**Before running**, after the fetch: no `main` on the remote (`no-branch`), no root `Cargo.toml` (`not-a-crate`),
or no `.cargo/mutants.toml` (`no-filters`) skip the visit with that reason and publish nothing.

**Unreachable:** "repository not found" or "permission denied" (deleted, renamed away, key revoked) is logged once
per change of state and backed off straight to 6 hours; it recovers on its own.

### 2.7 Time is injected

Every clock read, sleep and deadline goes through a trait (stern4rust's `injected-time`), so the loop, schedule,
back-off and timeouts are tested without sleeping.

### 2.8 Packaging

A plain binary, `sentinel4rust`: it is a service pointed at a list, not a command run inside a project.

## 3. Configuration

```toml
# ~/.config/sentinel4rust/roster.toml
work_dir = "/home/sentinel/sentinel4rust"   # clones, worktrees, cargo-mutants output
pause_seconds = 300                          # polling interval per repository
jobs = 6                                     # cargo-mutants -j
run_timeout_hours = 12
config_version = 1                           # bump to rerun every repository

[[repo]]
name = "twin4rust"
url = "git@github-twin4rust:umbgtt10/twin4rust.git"      # SSH host alias (5.2)

[[repo]]
name = "etheram-raft"
url = "git@github-etheram-raft:umbgtt10/etheram-raft.git"
branch = "main"                              # default
jobs = 2                                     # per-repository -j, e.g. tests not safe in parallel
```

The roster is read on start; edit it and restart the service. Each clone lives at `<work_dir>/repos/<name>`; if the
roster's `url` differs from the clone's `origin`, `origin` is updated.

Command line: `sentinel4rust --roster <file>` runs forever; `--once` visits every repository once and exits.

## 4. Architecture

A workspace like the other tools: `core/` (package `sentinel4rust`, library and binary) and `xtask/` (the gates).
Every dependency injected through the single constructor.

| trait | does |
|---|---|
| `CommandRunner` | runs a program in a directory with a deadline; exit code, stdout, stderr, or timeout |
| `Clock`, `Pause` | the current UTC time; waiting |
| `RepoSource` | fetches; reads `main`'s hash and the recorded `RunRecord` |
| `MutationRunner` | runs cargo-mutants at a hash; returns a `RunOutcome` |
| `ResultStore` | publishes a `RunRecord` on `mutants`, creating the orphan branch on first use |

| type | carries |
|---|---|
| `Roster`, `OnboardedRepo` | settings; name, URL, branch, jobs |
| `CommitHash`, `MutantKey` | a validated 40-hex hash; a position-free mutant identity |
| `MutantCounts` | caught, missed, timeout, unviable |
| `RunOutcome` | `Completed { counts, missed, timed_out }`, `BaselineFailed { log_tail }`, `ToolFailed { reason, log_tail }` |
| `RunRecord` | what `summary.toml` holds |
| `Schedule` | per repository: last visit, failures on the current commit, back-off deadline (in memory) |
| `RoundStep` | `Unchanged`, `Published`, `BackedOff`, `Unreachable`, `Skipped(reason)` |

Implementations: `SystemCommandRunner` (`std::process`, own process group), `SystemClock`, `ThreadPause`, `GitCli`
(`RepoSource` + `ResultStore`, shelling out to `git` with the system's SSH), `CargoMutantsCli`, `TomlRoster`.
`Sentinel::visit(&OnboardedRepo) -> RoundStep` does one visit; `run_forever` / `run_once` drive the `Schedule`.

Logging: one journal line per visit -- `repo=… hash=… step=… missed=… caught=… duration_s=…`.

## 5. The machine

A dedicated box: a `sentinel` user, `rustup` (each repository's `rust-toolchain.toml` is honoured), cargo-mutants
pinned with `cargo install cargo-mutants --version 27.1.0 --locked`, and a systemd unit
(`deploy/sentinel4rust.service`, `Restart=always`, `MemoryMax=` so a runaway test cannot freeze the box). The tool
takes a `flock` on `<work_dir>/sentinel4rust.lock`, so an accidental `--once` beside the service exits at once.

### 5.1 Security model

The tool reads every onboarded repository and writes exactly one branch in each: `mutants`. GitHub cannot scope a
credential to a branch, so this is two layers: a **deploy key with write access** per repository, and a
**ruleset** per repository that blocks every branch except `mutants`, with no bypass for deploy keys.

Accepted: a leaked key reads the repository; survivors and log tails are published on public repositories too;
the repositories' code (tests, build scripts, dependencies) runs where the keys are -- with the rulesets, a stolen
key can at worst write to `mutants`. Rulesets on private repositories need a paid GitHub plan; without one, those
rely on the tool pushing only `mutants` (2.4).

### 5.2 Setting it up

**On the machine, per repository**

- `ssh-keygen -t ed25519 -N "" -C "sentinel4rust <repo>" -f ~/.ssh/sentinel_<repo>` (keys `600`, `~/.ssh` `700`).
- Add a host alias to `~/.ssh/config` -- GitHub will not accept one deploy key on two repositories:

  ```
  Host github-<repo>
      HostName github.com
      User git
      IdentityFile ~/.ssh/sentinel_<repo>
      IdentitiesOnly yes
  ```

- Once per machine: `ssh-keyscan github.com >> ~/.ssh/known_hosts`, checked against GitHub's published
  fingerprints.
- Add `[[repo]]` to the roster with `url = "git@github-<repo>:umbgtt10/<repo>.git"`.

**On GitHub, per repository**

- Settings → Deploy keys → Add: `sentinel_<repo>.pub`, **Allow write access**.
- Settings → Rules → Rulesets → New branch ruleset `sentinel: only mutants`:
  - enforcement **Active**; target **all branches**, **exclude** `mutants`;
  - **Restrict creations**, **Restrict updates**, **Restrict deletions**, **Block force pushes**;
  - bypass list: yourself (or Repository admin) only -- **no deploy keys**.
- Leave `mutants` without rules. If you require signed commits, require them on `main` only.

**Verify, as `sentinel`**

- `git ls-remote git@github-<repo>:umbgtt10/<repo>.git` works.
- `sentinel4rust --roster <file> --once` creates and pushes `mutants`.
- These are all rejected: `git push origin HEAD:refs/heads/main`, `git push origin HEAD:refs/heads/probe`,
  `git push origin --delete main`.

**Revoke / rotate:** delete the deploy key on GitHub and the key file; to rotate, add the new key, switch the
alias's `IdentityFile`, then delete the old one.

## 6. Testing

- **Unit, with fakes:** unchanged `main` publishes nothing; a moved `main` publishes once; the first run creates
  the branch; a `config_version` bump reruns; a first baseline failure reruns and a second publishes
  `baseline-failed`; tool failures back off, doubling to the cap, and the 3rd publishes `tool-failed`; a new
  commit resets the back-off; the run cap counts as a tool failure; the not-ready cases skip with their reason;
  "not found" is `Unreachable`, logged once.
- **Schedule, with a fake clock:** no repository visited twice within `pause_seconds`; sleeping only when none is
  due; `--once` visits each repository once.
- **Keys:** the same mutant keeps its key when lines are inserted above it; duplicate mutations get ordinals;
  new/gone are set differences; timeouts never count as new.
- **`SystemCommandRunner`:** a command past its deadline is killed with its children.
- **`GitCli` on real git, no network:** bare repositories with `file://` remotes -- fetching without `mutants`
  works; the orphan branch is created with its files; the record reads back; `main` is byte-identical after every
  publish; worktrees are gone after every run.
- **`CargoMutantsCli`:** arguments against a fake runner; the parser against `mutants.out` fixtures captured from
  real runs (survivors, baseline failure, timeout).
- **End to end (stage 2):** real cargo-mutants on a three-function fixture crate in a local bare repository,
  `--once`, asserting the published lists.
- The family's gates from the first commit: stern4rust, dry4rust, crap4rust, twin4rust, iceberg4rust,
  cargo-shear.

## 7. Implementation slices

Test-first, one commit per slice, both stages green before each.

| # | slice |
|---|---|
| S0 | workspace (`core/`, `xtask/`), CLAUDE.md, gates green on an empty library (family gates: see 9) |
| S1 | data model: `CommitHash`, `MutantKey`, `MutantCounts`, `RunOutcome`, `RunRecord` and `summary.toml`, `Roster` |
| S2 | contracts, `SystemCommandRunner` with deadlines, `SystemClock`, `ThreadPause` |
| S3 | `GitCli` read side: fetch with or without `mutants`, `main`'s hash, the record |
| S4 | `GitCli` publish side: orphan branch, overwrite, commit, push only `mutants` |
| S5 | `CargoMutantsCli`: arguments, `outcomes.json` parser and keys, exit codes checked against 27.1.0, baseline retry |
| S6 | `Sentinel` and `Schedule`: visit, polling, back-off, `tool-failed`, skips, unreachable |
| S7 | command line, `main`, roster file, lock |
| S8 | end-to-end gate on a fixture crate; sentinel4rust onboarded on itself |
| S9 | `deploy/` (systemd unit, set-up notes from 5.2, a script running the verification pushes), README |

**Before S0:** run cargo-mutants by hand on the two or three largest repositories and note wall time, mutant count,
and peak disk and RAM. It settles `jobs`, the run cap, and whether the ideas below are needed, and supplies the real
`outcomes.json` for S5.

## 8. Ideas for later

Only if real use asks for them:

- **Incremental runs** (`--in-diff` against the last full record, with a periodic full run) if full runs take too
  long. `kind = "full"` is already in `summary.toml`.
- **Docs-only commits** republishing the previous lists instead of a full run.
- **Re-checking new survivors** once before publishing, if flaky tests make the lists noisy.
- **`--in-place`** with a warm `target/` instead of `-j` copies, if measurement favours it.
- **Roster reload** without a restart.
- **A heartbeat** or notification when survivors rise, if the commit messages on `mutants` turn out not to be
  enough.

## 9. Implementation notes

Where the code differs from revision 3, and why.

- **No dependencies.** The build machine had no crates.io access, and the tool needs little: `core/src/toml.rs`
  is a small TOML subset (comments, strings, integers, booleans, `[[repo]]`), enough for the roster and
  `summary.toml`. `rust-version` is 1.89 for `File::try_lock`.
- **Keys from the `.txt` lists (2.3).** cargo-mutants already writes each mutant as
  `file:line:col: description`, and the description names the function. Stripping the position gives the key
  without depending on `outcomes.json`'s field names, which could not be checked against 27.1.0. Duplicates are
  numbered per file, not per function.
- **Counts from the lists.** `caught`, `missed`, `timeout` and `unviable` are the line counts of the matching
  files in `mutants.out`; a completed exit without `caught.txt` and `missed.txt` is a tool failure.
- **Baseline retry in `Sentinel`**, not in `CargoMutantsCli`: the runner reports one run; the policy lives with the
  rest of the policy.
- **Publishing without a worktree (2.4).** `GitCli` writes blobs, a tree in a temporary index and a commit with
  plumbing (`hash-object`, `update-index`, `write-tree`, `commit-tree`), then pushes
  `<commit>:refs/heads/mutants`. Nothing is checked out for publishing.
- **Bare clones** at `<work_dir>/repos/<name>.git`; remote branches fetched into `refs/remotes/origin/*`.
- **Cleanup on start**: `runs/` and `stage/` are emptied and `git worktree prune` runs in every clone.
- **The family's gates** (stern4rust, dry4rust, crap4rust, twin4rust, iceberg4rust, cargo-shear) were not
  available where this was written. `cargo xtask stage1` runs `fmt --check`, `clippy -D warnings` (pedantic) and
  the tests; add the family gates to `xtask/src/main.rs`.
- **Stage 2** is `real_cargo_mutants_on_a_fixture_crate` in `core/tests/end_to_end.rs`, `#[ignore]`d unless run with
  `cargo xtask stage2`. It has not been run yet: it is the first thing to run on the machine, and it confirms the
  exit codes and the `.txt` format against the pinned cargo-mutants.
