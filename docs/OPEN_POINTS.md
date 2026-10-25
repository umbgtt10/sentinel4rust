# Open Points

Known gaps, deliberately recorded rather than silently assumed correct. Each entry says what was actually observed
-- in the code at `48c8fb1`, in cargo-mutants 27.1.0's source, or on the development laptop -- or, where it could not
be observed yet, that it is to be verified on the machine. Recorded 2026-10-24. "Design N" names a section of
`docs/history/[SUPERSEDED] DESIGN.md`.

| # | point | weight |
|---|---|---|
| 1 | cargo-mutants builds in a private `/tmp`, which is usually RAM | closed: `TMPDIR` under `work_dir` |
| 2 | `cargo install sentinel4rust` installs the placeholder that does nothing | closed: `deploy/SETUP.md` |
| 3 | a large repository can hold the machine for 36 hours per commit and publish nothing | before onboarding a large repository |
| 4 | an unreadable `summary.toml` silently reruns the same commit forever | closed: its own state |
| 5 | every repository's code runs where every repository's key is | accepted in design 5.1; decide again |
| 6 | `cargo xtask` does not exist | closed: `.cargo/config.toml` |
| 7 | the family's gates and coding standards do not apply here yet | closed: `docs/ROADMAP.md` |
| 8 | the design and the code disagree in four places | small fix |
| 9 | only the `completed` outcome has been seen against the real cargo-mutants | to verify on the machine |
| 10 | repositories the tool cannot test yet | before onboarding each repository |
| 11 | flaky tests and machine-dependent timeouts make the lists noisy | to watch |
| 12 | a dead machine looks like a quiet repository | to decide |
| 13 | `config_version` has to be bumped by hand after a cargo-mutants upgrade | closed: the version counts |
| 14 | `verify-access.sh --include-main` force-pushes `main` | closed: `deploy/verify-access.sh` |
| 15 | what public repositories publish | accepted in design 5.1; decide per repository |
| 16 | smaller things | low |

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

**To close:** measure the largest repositories on the machine before onboarding them; size `run_timeout_hours` and
`jobs` from that, or shard them (design 8, incremental runs). Observed: the code paths. Not measured: the run times.

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

**To decide.** Observed: the layout in design 5.2 and the unit file.

## 6. `cargo xtask` does not exist

**Closed.** `.cargo/config.toml` defines `[alias] xtask = "run --quiet --package xtask --"`, and the gates are run as
`just stage1` and `just stage2`, which call it.

## 7. The family's gates and coding standards do not apply yet

**Closed.** All six family gates run in stage 2 with nothing baselined, beside the end-to-end gate; the code is in
the family's layout, one object per file, every test in a mirrored file under `core/tests/`; the docs are
ARCHITECTURE, the ADRs, ROADMAP, IMPLEMENTED-FEATURES and CHANGELOG. `docs/ROADMAP.md` lists the slices.

## 8. The design and the code disagree in four places

- **2.4** says publishing uses a second worktree and `git push origin HEAD:refs/heads/mutants`; section 9 and
  `git.rs` publish with plumbing and push `<commit>:refs/heads/mutants`, with no worktree.
- **2.6** says the exit codes were "checked against 27.1.0 in S5"; section 9 says `outcomes.json` could not be
  checked against 27.1.0. The exit-code table does match 27.1.0's `exit_code.rs` (read on 2026-10-23); the
  `outcomes.json` format was never needed, since the lists are read from the `.txt` files.
- **6** says the parser is tested "against `mutants.out` fixtures captured from real runs"; the tests in
  `mutants.rs` write synthetic lists through a fake runner, and no captured `mutants.out` is in the repository.
- **9** says stage 2 "has not been run yet". It ran and passed on the development laptop on 2026-10-24, against
  cargo-mutants 27.1.0.

Section 7 plans one commit per slice, test-first. The implementation arrived uncommitted and went in as one
commit, `48c8fb1`, so no slice can be reverted alone, and the history cannot show that the tests came first.

**To close:** update sections 2.4, 2.6, 6 and 9. Observed.

## 9. Only `completed` has been seen against the real cargo-mutants

Stage 2 (`real_cargo_mutants_on_a_fixture_crate`) runs the real cargo-mutants on a three-function crate and sees a
`completed` run. Baseline failure (exit `4`), timeouts (exit `3`), the run cap and a cargo-mutants usage error are
tested only through fakes and a stub. The mapping matches the 27.1.0 source, but the `.txt` lists' format under
`timeout.txt`, and the log tail on a real baseline failure, have not been seen.

**To verify on the machine:** a fixture whose tests fail (exit `4`), and one with a mutant that loops (exit `3`).

## 10. Repositories the tool cannot test yet

Each of these makes the run fail as `baseline-failed` or `tool-failed`, and each is to be verified before the
repository is onboarded:

- **Submodules:** `git worktree add` checks out no submodules.
- **Path dependencies outside the repository** (`path = "../sibling"`): the run's checkout has no sibling.
- **Private git dependencies:** the keys are reachable only through the per-repository host aliases
  (`github-<repo>`), and a dependency's URL names `github.com`. cargo's built-in git does not read `~/.ssh/config`
  at all (it authenticates through an SSH agent), and with `net.git-fetch-with-cli` the system git still finds no
  key for `github.com`.
- **Embedded targets:** a `.cargo/config.toml` with `build.target = "thumbv…"` (etheram-embassy and its consumers)
  cannot run tests on the host; such crates must be excluded in `.cargo/mutants.toml`, or the repository left out.
- **Pinned toolchains:** each repository's `rust-toolchain.toml` (etheram-core pins 1.99.1) has to be installable by
  rustup on the machine; whether the machine's rustup installs it on first use, or needs `rustup toolchain install`,
  is to be checked (rustup 1.29.1 on the development laptop).

## 11. Flaky tests and machine-dependent timeouts make the lists noisy

A flaky test that fails by chance marks a mutant `caught` that was not, hiding a survivor; one that fails in the
baseline triggers the retry (design 2.6) and, twice in a row, `baseline-failed`. etheram-ibft's base has a known
restart-test flake. Timeouts scale with the machine's speed, so the same commit can produce different `timeout.txt`
lists on different machines.

**To watch:** the first weeks of `git log -p mutants` on each repository. Design 8 lists re-checking new survivors
once before publishing, if it gets noisy.

## 12. A dead machine looks like a quiet repository

Nothing reports that the machine is alive. If it stops, every `mutants` branch simply stops moving, which looks the
same as "nothing changed on `main`". `summary.toml` carries `finished`, so staleness can be read, but only by someone
who looks.

**To decide:** a heartbeat (design 8), or a check that compares each repository's `main` with its `mutants` record
and says which are behind and by how long.

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

On a public repository, `mutants` is public: the survivor lists -- a map of the code no test pins -- and, on
failures, up to 200 lines of log tail with paths such as `/home/sentinel/...`. Design 5.1 accepts this. For
etheram-ibft, whose untested paths sit close to what is kept private, onboarding is a decision of its own. Commits on
`mutants` are unsigned, from `sentinel4rust@localhost`; a repository that requires signed commits on every branch
rejects them (design 5.2 asks for that rule on `main` only).

**To decide per repository.**

## 16. Smaller things

- **Ordinals can shift.** Duplicate keys in a file are numbered in source order (2.3); removing the first of two
  renumbers the second, which then reads as one gone and one new.
- **Unbounded capture.** `SystemCommandRunner` keeps a run's whole stdout and stderr in memory until it ends, then takes
  the tail. cargo-mutants writes little to stdout, so this is unlikely to matter, but a 12-hour run is not bounded.
- **Unreachable is matched in English.** `UNREACHABLE_MARKERS` (`git.rs`) are substrings of git's and GitHub's
  messages, with `LC_ALL=C`; if GitHub rewords them, an unreachable repository is treated as a transient failure and
  backs off by doubling instead of going straight to 6 hours. A changed host key is transient too.
- **`kill` is an external program.** `kill_group` shells out to `kill -KILL -- -<pgid>`; it is on every Linux, but the
  tool is Linux-only for it, as for `process_group`.
- **The default `jobs` is 1** in `roster.rs`; design section 3 and `deploy/roster.example.toml` show 6. Not wrong,
  but a roster without `jobs` runs serially.
