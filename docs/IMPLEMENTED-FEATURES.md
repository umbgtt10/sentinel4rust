# Implemented Features

What `sentinel4rust` does today. For why, see [ADRs/](ADRs/README.md); for how, [ARCHITECTURE.md](ARCHITECTURE.md);
for what changed, [CHANGELOG.md](../CHANGELOG.md); for what is known to be missing, [OPEN_POINTS.md](OPEN_POINTS.md).

## Unreleased (0.1.0)

### The loop

- Visits the roster's repositories forever, one at a time; no repository twice within `pause_seconds`; the longest
  waiting due repository first, ties in roster order; sleeps only when none is due, never less than a second
- `--once` visits every repository once, in roster order, and exits 1 if any visit backed off or found a repository
  unreachable
- A visit fetches, compares `main` with the record on `mutants`, and runs only when the record is not current: a
  different commit, a different `config_version`, or another cargo-mutants than the installed one (read once on
  start; when it cannot be read, it decides nothing)

### Repositories

- One full bare clone per repository at `<work_dir>/repos/<name>.git`, its origin reset when the roster's URL changes
- Explicit refspecs for the configured branch and, when the remote has it, `mutants`; a deleted `mutants` drops the
  record
- Skips, publishing nothing: `no-branch` (the branch is not on the remote), `not-a-crate` (no root `Cargo.toml` at
  `head`), `no-filters` (no `.cargo/mutants.toml` at `head`), `path-outside-repo` (a manifest at `head` has a
  `path` dependency outside the repository; the journal names the manifest), `toolchain-missing` (rustc does not
  run in the checkout; the journal quotes rustup's message)
- Git runs without prompts, in the C locale, with the system's SSH configuration

### Runs

- cargo-mutants in a detached worktree at `head` (`<work_dir>/runs/<name>`), submodules initialised, output at `<work_dir>/out/<name>` and
  `TMPDIR` at `<work_dir>/tmp/<name>`, all removed after every run; `--jobs` and the run cap per repository or from
  the roster
- `cargo mutants --version` and `rustc --version` recorded with every run
- New survivors are run once more on exactly their names, in the same checkout, before publishing; one caught then
  counts as caught
- Exit 0, 2 and 3 publish `completed`, with counts and lists read from `mutants.out`; exit 4 reruns once at once and,
  a second time, publishes `baseline-failed`; anything else backs off, and the third tool failure on one commit
  publishes `tool-failed`
- Every command has a deadline and is killed with its whole process group: git talking to the remote 5 minutes,
  other git 1 minute, the run `run_timeout_hours`
- Failing runs keep the last 200 lines and at most 16 KiB of their output

### Publishing

- An orphan `mutants` branch: `README.md`, `summary.toml`, `missed.txt`, `timeout.txt` (when any), `located/`
- Mutants keyed without line numbers, duplicates numbered in source order; `new` and `gone` against the previous
  record
- Commits written with plumbing only, as `sentinel4rust <sentinel4rust@localhost>`, and pushed as
  `<commit>:refs/heads/mutants`; nothing else is ever pushed
- A rejected push backs off; the next visit reruns

### Failures and back-off

- Back-off after a failure: `pause_seconds × 2^(failures-1)`, capped at 6 hours, reset by any success, unchanged
  `main` or skip
- A repository that is gone or refuses the key backs off to the cap at once and is logged once per change of state
- A `summary.toml` that does not parse is never run over: `step=unreadable` with the reason, logged once per change
  of state, backed off to the cap; the repository recovers when the record reads again
- Schedule and back-off are kept in memory; a restart costs at most an early visit

### Start-up and the command line

- `sentinel4rust --roster <file> [--once]`, `--help`, `--version`; a bad command line exits 2 with the usage
- `--status`: one read-only pass, one line per repository -- current, behind, no-record, unreadable, skipped or
  unknown, both commits, how long `main`'s head has waited -- exiting 1 when one has waited longer than
  `stale_after_hours` or could not be judged; its own clones and lock under `<work_dir>/status`
- The roster is validated on start, failing with the line or the key at fault; a second instance on the same work
  directory fails on its lock
- Leftovers of a crash are removed on start: checkouts, staging areas, stale worktree entries, temporary files
- One journal line on start and one per visit on stdout, for systemd's journal

### Deployment

- `deploy/sentinel4rust-mutants@.service` and `deploy/50-sentinel4rust-mutants.rules`: with `run_unit` in the roster,
  cargo-mutants runs in a unit of its own that cannot read `~/.ssh`, started and stopped through a polkit rule allowing
  only that; `deploy/verify-isolation.sh` proves it on the machine
- `deploy/sentinel4rust.service`, a systemd unit, which ends a run in progress in the run unit when it stops or
  restarts; `deploy/SETUP.md`, the machine, keys and rulesets step by step;
  `deploy/verify-access.sh`, checking that a repository's key reads and that its ruleset refuses every branch but
  `mutants`; `deploy/roster.example.toml`
- `deploy/PROVISION.md`, the set-up for an agent on the machine, through three idempotent scripts with `--dry-run`:
  `deploy/provision.sh` (the machine, the `sentinel` user and its tools, sentinel4rust from the checkout, the units,
  the isolation proof), `deploy/onboard.sh` (per repository the key, the ruleset in `deploy/rulesets/`, the deploy key
  and the access proof, changing GitHub only with `--apply-github`) and `deploy/go-live.sh` (the first pass, then the
  service); `deploy/status-check.sh` watches it from another machine. Proven on the machine only once run there (open
  point 17)

### Tested on itself

- `.cargo/mutants.toml` makes sentinel4rust a repository the service can test: the core is mutated, `xtask/` left out

### No dependencies

- The core uses the standard library alone, with its own TOML subset for the roster and `summary.toml`
