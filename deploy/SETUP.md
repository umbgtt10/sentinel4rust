# Setting up the machine

In order. Why it is set up this way: `docs/ADRs/ADR-DeployKeyAndRuleset.md`.

The automated path is [`PROVISION.md`](PROVISION.md): an agent on the machine runs `deploy/provision.sh`,
`deploy/onboard.sh` and `deploy/go-live.sh`, each idempotent and with `--dry-run`, and asks the owner before anything
changes on GitHub. This file stays the reference of what each step does, by hand.

## 1. User and tools

```sh
sudo useradd --create-home --shell /bin/bash sentinel
sudo -iu sentinel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
cargo install cargo-mutants --version 27.1.0 --locked
cargo install --path /path/to/sentinel4rust/cli --locked       # from the repository; the release: cargo install sentinel4rust --locked
mkdir -p ~/.config/sentinel4rust && cp deploy/roster.example.toml ~/.config/sentinel4rust/roster.toml
ssh-keyscan github.com >> ~/.ssh/known_hosts                     # check against GitHub's published fingerprints
```

`deploy/provision.sh` does this section, and also: the packages; unattended security upgrades, rebooting at 04:30
when one needs it; the time sync check; Rust 1.99.1 as the default toolchain, with rustfmt, clippy and llvm-tools;
sentinel4rust built from a `git archive` of the checkout's HEAD into `~sentinel/src/sentinel4rust`; a roster
skeleton instead of the example, never overwritten; and GitHub's host keys refused unless each fingerprint is one
GitHub publishes.

## 2. Per repository

On the machine, as `sentinel`:

```sh
ssh-keygen -t ed25519 -N "" -C "sentinel4rust <repo>" -f ~/.ssh/sentinel_<repo>
chmod 700 ~/.ssh && chmod 600 ~/.ssh/sentinel_*
cat >> ~/.ssh/config <<EOF
Host github-<repo>
    HostName github.com
    User git
    IdentityFile ~/.ssh/sentinel_<repo>
    IdentitiesOnly yes
    StrictHostKeyChecking yes
EOF
```

On GitHub:

- Settings → Deploy keys → Add: `~/.ssh/sentinel_<repo>.pub`, **Allow write access**.
- Settings → Rules → Rulesets → New branch ruleset `sentinel: only mutants`: enforcement **Active**; target
  **all branches**, **exclude** `mutants`; **Restrict creations**, **Restrict updates**, **Restrict deletions**,
  **Block force pushes**; bypass list: the **Repository admin** role only -- on a personal repository, you --,
  **no deploy keys**. The same ruleset as the API's body: `deploy/rulesets/sentinel-only-mutants.json`.
- Settings → Rules → Rulesets → New tag ruleset `sentinel: no tags`: enforcement **Active**; target **all tags**;
  **Restrict creations**, **Restrict updates**, **Restrict deletions**; the same bypass list. A write key can write
  every ref, tags included, and the branch ruleset does not cover them. As the API's body:
  `deploy/rulesets/sentinel-no-tags.json`.
- The repository needs a `.cargo/mutants.toml` on `main` (it is skipped without one).

Verify, then add it to the roster:

```sh
deploy/verify-access.sh git@github-<repo>:umbgtt10/<repo>.git
```

It tries to create a branch and a tag, and both must be rejected. `deploy/onboard.sh <repo>...` does this section,
the rulesets before the deploy key, and appends the roster entry once the access proof passes. It changes GitHub
only with `--apply-github`, and refuses a private repository on the free plan, on which GitHub offers no ruleset.

## 3. The service

```sh
sudo cp deploy/sentinel4rust.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo -iu sentinel sentinel4rust --roster ~/.config/sentinel4rust/roster.toml --once   # first pass by hand
sudo systemctl enable --now sentinel4rust
journalctl -u sentinel4rust -f
```

The `--once` pass and the service share a lock: stop the service before a manual pass. `deploy/go-live.sh` makes
the pass -- in a transient unit with the service's user and limits, so that it outlives the shell that started it --
and enables the service only if no visit failed.

## 4. Sizing a repository's run

Before onboarding a large repository, measure one full run on the machine, as `sentinel`:

```sh
git clone <url> /tmp/measure && cd /tmp/measure
cargo mutants --list | wc -l                                          # mutants, after .cargo/mutants.toml's filters
/usr/bin/time -v cargo mutants --jobs 6 --output /tmp/measure-out     # "Elapsed (wall clock) time", "Maximum resident set size"
```

- **`jobs`:** every job builds its own copy of the tree under `TMPDIR` (for the service, `<work_dir>/tmp/<name>`, on
  the work directory's disk). Raise it while cores, memory and disk allow; set it on the repository's `[[repo]]` when
  its tests are not safe in parallel or its builds are heavier than the rest.
- **`run_timeout_hours`:** the measured wall time with a margin, on the repository's `[[repo]]` when it differs from
  the roster's. A run that hits the cap is a tool failure and the third on one commit publishes `tool-failed`, so a
  cap too short costs three full runs per commit, during which nothing else is visited.
- **Tests in another package:** when `.cargo/mutants.toml` names `test_package`, cargo-mutants times its baseline
  without them and sets the test timeout too short for them (`docs/OPEN_POINTS.md` point 10): set
  `minimum_test_timeout` there, above those tests' run, and check `test_timeout_seconds` in the first `summary.toml`.
- Note the mutant count, wall time and peak memory beside the repository's roster entry.

## 5. Watching it from elsewhere

A stopped sentinel looks like repositories that did not move. `--status` makes one read-only pass -- it fetches,
runs nothing and pushes nothing -- and prints one line per repository: `state=current`, `behind`, `no-record`,
`unreadable`, `skipped` or `unknown` (it could not fetch), `main`'s and the record's commits, and, when the record
does not answer for `main`, how long `main`'s head has waited for one. A record whose timeouts outnumber its caught
mutants adds `warning=timeouts-outnumber-caught`. It exits 1 when one has waited longer than the roster's
`stale_after_hours` (24 by default), could not be judged or has such a record, so a timer or cron job on another
machine with read access can mail on failure:

```sh
sentinel4rust --roster roster.toml --status
```

It keeps its own clones and lock under `<work_dir>/status`, so it can also run beside the service.
`deploy/status-check.sh` wraps it for cron or a systemd user timer: quiet while nothing needs attention, otherwise
the lines printed and, with `--mail`, mailed. `PROVISION.md` has a roster, a cron line and the timer.

## 6. Running cargo-mutants without the keys

Every repository's build scripts, tests and dependencies run where cargo-mutants runs. To keep them away from the
deploy keys, cargo-mutants can run in a unit of its own, `sentinel4rust-mutants@.service`, with
`InaccessiblePaths=/home/sentinel/.ssh`; git, which needs the keys, stays in the service. The service starts the unit,
and stops it when the service stops, through a polkit rule that lets the `sentinel` user start and stop that unit's
instances and nothing else.

```sh
sudo cp deploy/sentinel4rust-mutants@.service /etc/systemd/system/
sudo cp deploy/50-sentinel4rust-mutants.rules /etc/polkit-1/rules.d/
sudo systemctl daemon-reload
sudo -iu sentinel /path/to/sentinel4rust/deploy/verify-isolation.sh /home/sentinel/sentinel4rust
```

Then add `run_unit = "sentinel4rust-mutants"` to the roster and restart the service. Each cargo-mutants command is
written to `<work_dir>/jobs/run/job.toml`, the unit runs it with `sentinel4rust --run-job` and writes the result and
output beside it, and the service reads them back. The units' paths assume `work_dir = /home/sentinel/sentinel4rust`;
change the run unit's `ExecStart=` and the service's `SENTINEL4RUST_LOCK` if yours differs. This has not yet been
proven on the machine: `verify-isolation.sh` is that proof, of the stop as well. `deploy/provision.sh` installs the
units and the rule, with drop-ins when the work directory differs, runs the proof, and adds `run_unit` to the roster
only if it passes.

## 7. Deciding what a public repository publishes

On a public repository, `mutants` is public too: the survivor lists -- a map of the code no test pins -- and, on
failures, up to 200 lines of log tail, with paths such as `/home/sentinel/...`. Decide per repository, before adding
it to the roster, whether that map may be public; a repository whose survivors should not be is left off the roster,
and nothing about it is published. Record the decision beside its roster entry, or in the roster's comments when it
is left off.

Commits on `mutants` are unsigned, from `sentinel4rust@localhost`: a repository requiring signed commits on every
branch rejects them, so require them on `main` only (section 2's ruleset leaves `mutants` without rules).
