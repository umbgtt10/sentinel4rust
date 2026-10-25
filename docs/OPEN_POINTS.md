# Open Points

Known gaps, deliberately recorded rather than silently assumed correct. Each entry says what was actually observed
-- in the code at `48c8fb1`, in cargo-mutants 27.1.0's source, or on the development laptop -- or, where it could not
be observed yet, that it is to be verified on the machine. Recorded 2026-10-24. "Design N" names a section of
`docs/history/[SUPERSEDED] DESIGN.md`.

| # | point | weight |
|---|---|---|
| 1 | cargo-mutants builds in a private `/tmp`, which is usually RAM | closed: `TMPDIR` under `work_dir` |
| 2 | `cargo install sentinel4rust` installs the placeholder that does nothing | closed: `deploy/SETUP.md` |
| 3 | a large repository can hold the machine for 36 hours per commit and publish nothing | to measure on the machine |
| 4 | an unreadable `summary.toml` silently reruns the same commit forever | closed: its own state |
| 5 | every repository's code runs where every repository's key is | closed in the code; prove on the machine |
| 6 | `cargo xtask` does not exist | closed: `.cargo/config.toml` |
| 7 | the family's gates and coding standards do not apply here yet | closed: `docs/ROADMAP.md` |
| 8 | the design and the code disagree in four places | closed: `docs/DESIGN.md` |
| 9 | only the `completed` outcome has been seen against the real cargo-mutants | closed: stage-2 fixtures |
| 10 | repositories the tool cannot test yet | before onboarding each repository |
| 11 | flaky tests and machine-dependent timeouts make the lists noisy | re-check done; to watch |
| 12 | a dead machine looks like a quiet repository | closed: `--status` |
| 13 | `config_version` has to be bumped by hand after a cargo-mutants upgrade | closed: the version counts |
| 14 | `verify-access.sh --include-main` force-pushes `main` | closed: `deploy/verify-access.sh` |
| 15 | what public repositories publish | closed: decided per repository |
| 16 | smaller things | closed: each fixed or recorded |
| 17 | the automated set-up is proven only as far as the laptop allows | to prove on the machine |
| 18 | a private repository on GitHub's free plan cannot be onboarded | open: the owner decides |
| 19 | restarting the service leaves a run in the run unit running | open: a fix proposed |
| 20 | the ruleset covers branches, not tags | open: the owner decides |
| 21 | the deploy keys depend on the GitHub CLI's authorization | recorded |

---

## 1. cargo-mutants builds in a private `/tmp`, which is usually RAM

**Closed.** cargo-mutants copies the tree once per `--jobs` through the `tempfile` crate, so the copies went to the
service's private `/tmp`, a `tmpfs` on many machines, where they count against `MemoryMax=`. `CargoMutantsCli` now
gives the run `TMPDIR=<work_dir>/tmp/<name>`, on the same disk as the clones, creates it before the run and removes
it after; copies a killed run leaves behind are removed on start with the other leftovers. The machine's `/tmp` no
longer matters.

## 2. `cargo install sentinel4rust` installs the placeholder

**Closed.** crates.io has only `0.0.1`, the placeholder that prints "under construction" and exits `0`.
`deploy/SETUP.md` now installs from the repository only, with `cargo install --path core --locked`, until `0.1.0`
is published; nothing is published from here.

## 3. A large repository can hold the machine for 36 hours per commit

`run_timeout_hours` defaults to 12 (`roster.rs`), and `MAX_TOOL_FAILURES` is 3 per commit (`schedule.rs`): a run that
needs more than 12 hours is killed three times -- up to 36 hours, during which nothing else is visited, since one
repository runs at a time -- and then publishes `tool-failed`. When `main` moves, the count starts again.
etheram-ibft's core packages alone have 2,357 mutants (counted with `--list` on 2026-10-23); their run time is not
known.

Design section 7 asks for cargo-mutants to be run by hand on the two or three largest repositories before S0, to set
`jobs` and the run cap. No measurement is recorded in the repository. The only measured run is twin4rust's: 215
mutants, 3 min 20 s, `-j 1`, on the development laptop.

**Code side closed.** `run_timeout_hours` can be set per `[[repo]]`, defaulting to the roster's, beside the
per-repository `jobs`, and `deploy/SETUP.md` section 4 says how to measure a repository and size both from it.
**Still open:** the real run times, to be measured on the machine before each large repository is onboarded.

## 4. An unreadable `summary.toml` silently reruns the same commit

**Closed.** `GitCli::sync` read the record with `.ok()`, so a `summary.toml` that did not parse read as no record and
the full run started again on every visit, saying nothing. The snapshot now carries a `RecordState`: absent (no
`mutants`, no `summary.toml`, or a schema this version does not know, which is run again as before), present, or
unreadable with its `RecordError`. An unreadable record is never run over -- it may be one a hand edit or a newer
tool broke, and overwriting it would lose it --: the visit is `step=unreadable` with the reason, logged once per
change of state like `unreachable`, backed off straight to the 6-hour cap, and the repository recovers by itself
when the record reads again.

## 5. Every repository's code runs where every repository's key is

All deploy keys live in `~sentinel/.ssh`, and cargo-mutants runs each onboarded repository's build scripts, tests
and dependencies as the same user. One compromised dependency in one repository can read every key: it can then read
every onboarded repository, private ones included, and write every `mutants` branch. On private repositories without
a paid plan there is no ruleset (design 5.1), so a stolen write key can also push to `main` there.

Design 5.1 accepts this. It is recorded here because the blast radius covers the private repositories, and because it
can be narrowed: run git as one user, which holds the keys, and cargo-mutants as another, which cannot read them; or
give the run a mount namespace without `~/.ssh` (systemd `InaccessiblePaths=` on a separate unit).

**Closed in the code; to prove on the machine.** With `run_unit = "sentinel4rust-mutants"` in the roster, every
cargo-mutants command runs in `sentinel4rust-mutants@run.service`, a fixed system template unit (`deploy/`) with
`InaccessiblePaths=/home/sentinel/.ssh`, which the service starts through a polkit rule allowing the `sentinel` user
to start that unit's instances and nothing else; git, which needs the keys, stays in the service. The command goes
in as a job file and the result and output come back beside it (`core/src/system/unit/`). Tested here through a
stand-in for systemd; the unit and the rule have not been installed on the machine, and `deploy/verify-isolation.sh`
-- `~/.ssh` readable in the service, refused in the unit -- is the proof still to run there. `deploy/provision.sh`
runs it after installing the unit and the rule, and sets `run_unit` only when it passes (point 17).

## 6. `cargo xtask` does not exist

**Closed.** `.cargo/config.toml` defines `[alias] xtask = "run --quiet --package xtask --"`, and the gates are run as
`just stage1` and `just stage2`, which call it.

## 7. The family's gates and coding standards do not apply yet

**Closed.** All six family gates run in stage 2 with nothing baselined, beside the end-to-end gate; the code is in
the family's layout, one object per file, every test in a mirrored file under `core/tests/`; the docs are
ARCHITECTURE, the ADRs, ROADMAP, IMPLEMENTED-FEATURES and CHANGELOG. `docs/ROADMAP.md` lists the slices.

## 8. The design and the code disagree in four places

**Closed.** The design moved, unchanged, to `docs/history/`; the documents that replace it -- `docs/ARCHITECTURE.md`
and the ADRs -- were written from the code and say what holds in the four places: publishing by plumbing and
`<commit>:refs/heads/mutants` (2.4), the exit codes as read in 27.1.0's source, with `outcomes.json` never read
(2.6), the parser tested against lists captured from real runs since open point 9 (6), and stage 2 having run (9).
The stub at `docs/DESIGN.md` lists the four, so a reader of the history copy is told where it was wrong.

Section 7 planned one commit per slice, test-first; the implementation went in as one commit, `48c8fb1`, so its
slices cannot be reverted alone. Everything since is one commit per slice.

## 9. Only `completed` has been seen against the real cargo-mutants

**Closed.** Stage 2 now runs the real cargo-mutants 27.1.0 on three fixture crates: survivors (exit 2), a crate whose
test fails without any mutation (exit 4, published as `baseline-failed` with the real log tail, after the one retry),
and `spin`, whose `+ with *` mutant loops (exit 3, `timeout.txt` holding it). Seen on 2026-10-25: on exit 4
cargo-mutants still writes all four lists, empty; on exit 3 `timeout.txt` uses the same `file:line:col: description`
lines as the others. The lists of those two runs are kept, unedited, in `core/tests/fixtures/mutants_out/`, and stage
1 reads them. The run cap and a cargo-mutants usage error remain tested through fakes and the stub.

## 10. Repositories the tool cannot test yet

Each of these makes the run fail as `baseline-failed` or `tool-failed`, and each is to be verified before the
repository is onboarded:

- **Submodules:** closed -- a commit with a `.gitmodules` has its submodules initialised in the run's worktree,
  recursively (`git submodule update --init --recursive`); a submodule git cannot fetch is a checkout failure that
  backs off, and private submodules meet the same keys problem as private git dependencies below.
- **Path dependencies outside the repository** (`path = "../sibling"`): closed -- readiness reads every
  `Cargo.toml` at `head` and skips the visit with `path-outside-repo`, naming the manifest, when a `path` leaves the
  repository; the run's checkout would have no sibling.
- **Private git dependencies:** the keys are reachable only through the per-repository host aliases
  (`github-<repo>`), and a dependency's URL names `github.com`. cargo's built-in git does not read `~/.ssh/config`
  at all (it authenticates through an SSH agent), and with `net.git-fetch-with-cli` the system git still finds no
  key for `github.com`.
- **Embedded targets:** a `.cargo/config.toml` with `build.target = "thumbv…"` (etheram-embassy and its consumers)
  cannot run tests on the host; such crates must be excluded in `.cargo/mutants.toml`, or the repository left out.
- **Pinned toolchains:** closed in the code -- `rustc --version`, run in the checkout for the record, failing means
  the toolchain its `rust-toolchain.toml` names is not there: the visit is skipped with `toolchain-missing` and
  rustup's message, not counted as a tool failure, and cargo-mutants is not started. Whether the machine's rustup
  installs a pinned toolchain on first use or needs `rustup toolchain install` is still to be checked there (rustup
  1.29.1 on the development laptop).

Private git dependencies and embedded targets stay as recorded above: to be settled per repository before it is
onboarded.

## 11. Flaky tests and machine-dependent timeouts make the lists noisy

A flaky test that fails by chance marks a mutant `caught` that was not, hiding a survivor; one that fails in the
baseline triggers the retry (design 2.6) and, twice in a row, `baseline-failed`. etheram-ibft's base has a known
restart-test flake. Timeouts scale with the machine's speed, so the same commit can produce different `timeout.txt`
lists on different machines.

**Code side closed:** new survivors -- those whose keys the previous record did not list -- are run once more, in
the same checkout, with `cargo mutants --re` on exactly their names before anything is published; one caught on the
re-check counts as caught, one missed or timed out again stays missed. **To watch:** the first weeks of
`git log -p mutants` on each repository, and timeouts, which the re-check does not smooth.

## 12. A dead machine looks like a quiet repository

Nothing reports that the machine is alive. If it stops, every `mutants` branch simply stops moving, which looks the
same as "nothing changed on `main`". `summary.toml` carries `finished`, so staleness can be read, but only by someone
who looks.

**Closed.** `sentinel4rust --roster <file> --status` compares each repository's `main` with its `mutants` record in one
read-only pass and says, per repository, whether the record is current and, when not, how long `main`'s head has
waited; it exits 1 when one has waited longer than `stale_after_hours` (24 by default) or could not be judged. Run
from another machine with read access, a dead sentinel shows; `deploy/SETUP.md` section 5 says how.

## 13. `config_version` has to be bumped by hand

**Closed.** A record is current when its commit and `config_version` match and, when the installed version is known,
it was made by the installed cargo-mutants: `EntryPoint` reads `cargo mutants --version` once on start, and a record
whose `cargo_mutants_version` differs is run again, once per repository. When the installed version cannot be read,
it decides nothing, so a broken install does not rerun every repository on every visit.

## 14. `verify-access.sh --include-main` force-pushes `main`

**Closed.** Every write the script makes to `main` now carries a lease on what it last saw there: putting `main` back
expects what the probe left (the probe commit, or no `main` after the delete probe), and the delete probe expects
`main` where it was. A commit someone pushes to `main` in between is never overwritten or deleted: the lease refuses,
the script says so, and leaves `main` alone. `--include-main` fetches `main`'s whole history, since putting `main`
back after the delete probe is a push a remote refuses from a shallow clone. Checked by hand against local bare
repositories without and with a stand-in ruleset, and with a push landing on `main` right after the probe's.

## 15. What public repositories publish

**Closed.** On a public repository `mutants` is public: the survivor lists -- a map of the code no test pins -- and, on
failures, up to 200 lines of log tail with paths such as `/home/sentinel/...`. It is decided per repository, as
`deploy/SETUP.md` section 7 says, and recorded in `deploy/roster.example.toml`: twin4rust, etheram-raft and
sentinel4rust publish theirs -- sentinel4rust is tested on itself, its `.cargo/mutants.toml` leaving `xtask/` out --;
etheram-ibft stays off, by decision. Commits on `mutants` are unsigned, so signed commits are required on `main`
only.

## 16. Smaller things

**Closed: each fixed or recorded.**

- **Ordinals can shift** -- recorded, inherent to position-free keys. Duplicate keys in a file are numbered in source
  order; removing the first of two renumbers the second, which then reads as one gone and one new.
- **Unbounded capture** -- fixed. A command can keep only the tail of each output stream; cargo-mutants' runs keep
  their last 64 KiB, more than the 16 KiB log tail needs, so a 12-hour run no longer holds its whole output in
  memory. git's output is kept whole: it is short, and parsed.
- **Unreachable is matched in English** -- recorded. `UnreachableClassifier` matches substrings of git's and
  GitHub's messages, with `LC_ALL=C`; if GitHub rewords them, an unreachable repository is treated as a transient
  failure and backs off by doubling instead of going straight to 6 hours. A changed host key is transient too.
- **`kill` is an external program** -- recorded. `ProcessGroup` sends `kill -KILL -- -<pgid>`: it is on every Linux,
  and the tool is Linux-only for it, as for process groups; the alternative, `libc`, is a dependency
  (`ADR-NoDependencies`).
- **The default `jobs` is 1** -- fixed in the docs: `deploy/roster.example.toml` sets 6 and says so, and
  `docs/ARCHITECTURE.md` gives 1 as the default.

## 17. The automated set-up is proven only as far as the laptop allows

`deploy/PROVISION.md` and its four scripts (2026-10-25) were run on the development laptop, without root and without
changing GitHub. Proven there:

- `onboard.sh --local` against `file://` bare copies of iceberg4rust and twin4rust: keys, host aliases and roster
  entries made once, every step `ok` on a re-run. A `--once` pass over that roster then published iceberg4rust's 48
  survivors to its copy in 230 s and skipped twin4rust with `no-filters`.
- a read-only `onboard.sh --dry-run` against GitHub, for this repository: `BLOCKED` (point 18).
- the dry runs of `provision.sh` and `go-live.sh`, and their refusals, among them a GitHub host key whose fingerprint
  GitHub does not publish; the `git archive` build `provision.sh` makes; `run_unit` inserted before a roster's first
  `[[repo]]`.
- `systemd-run --wait --collect`, as `go-live.sh` uses it, passing a pass's exit code on, 0 and 1, in a user unit.
- `status-check.sh`: quiet while current, the lines with `--verbose`, exit 1 for a repository it cannot fetch and for
  a `main` three days without a record.

**To prove on the machine**, the first time each runs. Everything that needs root: the packages, the
unattended-upgrade settings and the reboot they schedule, the `sentinel` user, rustup and the toolchain installed
for it, the units, the polkit rule and with them `verify-isolation.sh` (point 5), the first pass in its transient
unit as `sentinel`, the service enabled. Everything that changes GitHub: the ruleset's body accepted as it is by
`POST /repos/{owner}/{repo}/rulesets`, the deploy key added, `verify-access.sh` rejected by a real ruleset. And
whether `sudo -n` sees the cache the owner's `sudo -v` left, which depends on sudo's `timestamp_type` and on how
Claude Code runs commands: `PROVISION.md` stops and asks when it does not.

## 18. A private repository on GitHub's free plan cannot be onboarded

GitHub offers rulesets on private repositories only on paid plans; without one it refuses even to list them -- HTTP
403, "Upgrade to GitHub Pro or make this repository public to enable this feature", seen on 2026-10-25 for this
repository, which is private. `ADR-DeployKeyAndRuleset` accepts such a repository relying on the tool alone to push
only `mutants`, but `verify-access.sh` -- creating a branch must be rejected -- cannot pass there, so `onboard.sh`
marks it `BLOCKED` and changes nothing on GitHub for it.

**Open, for the owner:** make the repository public, move to a paid plan, leave it off the roster, or accept it
without a ruleset. The last needs an `onboard.sh` option that skips the ruleset and the probe and says so in the
roster entry.

## 19. Restarting the service leaves a run in the run unit running

With `run_unit` set, cargo-mutants runs in `sentinel4rust-mutants@run.service`, which nothing ties to the service:
stopping or restarting `sentinel4rust` leaves it running. The new service prunes `runs/` and `tmp/` from under it on
start, and its first run rewrites `jobs/run/` and calls `systemctl start --wait` on the instance still active, which
waits for the old run and then reads the result the old run wrote as its own. Read in the code
(`UnitCommandRunner::run`, `GitCli::prune`), not seen on the machine. A reboot stops every unit, so it is not
affected.

**Until it is fixed:** restart only while `systemctl is-active sentinel4rust-mutants@run.service` says `inactive`
(`deploy/PROVISION.md`, "Restarting the service"). **Proposed:** `PartOf=sentinel4rust.service` in the run unit, so
that stopping or restarting the service stops the run first; to be decided, and proven on the machine.

## 20. The ruleset covers branches, not tags

`sentinel: only mutants` is a branch ruleset. A deploy key with write access can write every ref, so a leaked key can
create, move or delete tags, which no rule restricts; `verify-access.sh` probes branches only.

**Open, for the owner:** a second ruleset beside the first in `deploy/rulesets/`, target `tag`, every tag, creation,
update and deletion restricted to repository admins, created by `onboard.sh` and probed by `verify-access.sh`.

## 21. The deploy keys depend on the GitHub CLI's authorization

`onboard.sh` adds each deploy key through `gh`, and `gh repo deploy-key add --help` says such a key is tied to its
authentication token: de-authorizing the GitHub CLI app in GitHub's settings deletes the keys. `gh auth logout`, at
the end of `PROVISION.md`, only forgets the token on the machine.

**Recorded:** `PROVISION.md` tells the owner at the go. A key deleted this way fails every fetch (`unreachable` in
the service's journal), and from another machine shows only once `main` moves and its record falls behind;
`onboard.sh` puts it back. A key added by hand in GitHub's settings is tied to no token.
