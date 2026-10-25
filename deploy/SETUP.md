# Setting up the machine

In order. Why it is set up this way: `docs/ADRs/ADR-DeployKeyAndRuleset.md`.

## 1. User and tools

```sh
sudo useradd --create-home --shell /bin/bash sentinel
sudo -iu sentinel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
cargo install cargo-mutants --version 27.1.0 --locked
cargo install --path /path/to/sentinel4rust/core --locked      # from the repository, until 0.1.0 is on crates.io
mkdir -p ~/.config/sentinel4rust && cp deploy/roster.example.toml ~/.config/sentinel4rust/roster.toml
ssh-keyscan github.com >> ~/.ssh/known_hosts                     # check against GitHub's published fingerprints
```

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
EOF
```

On GitHub:

- Settings → Deploy keys → Add: `~/.ssh/sentinel_<repo>.pub`, **Allow write access**.
- Settings → Rules → Rulesets → New branch ruleset `sentinel: only mutants`: enforcement **Active**; target
  **all branches**, **exclude** `mutants`; **Restrict creations**, **Restrict updates**, **Restrict deletions**,
  **Block force pushes**; bypass list: yourself only, **no deploy keys**.
- The repository needs a `.cargo/mutants.toml` on `main` (it is skipped without one).

Verify, then add it to the roster:

```sh
deploy/verify-access.sh git@github-<repo>:umbgtt10/<repo>.git
```

## 3. The service

```sh
sudo cp deploy/sentinel4rust.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo -iu sentinel sentinel4rust --roster ~/.config/sentinel4rust/roster.toml --once   # first pass by hand
sudo systemctl enable --now sentinel4rust
journalctl -u sentinel4rust -f
```

The `--once` pass and the service share a lock: stop the service before a manual pass.

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
- Note the mutant count, wall time and peak memory beside the repository's roster entry.

## 5. Watching it from elsewhere

A stopped sentinel looks like repositories that did not move. `--status` makes one read-only pass -- it fetches,
runs nothing and pushes nothing -- and prints one line per repository: `state=current`, `behind`, `no-record`,
`unreadable` or `skipped`, `main`'s and the record's commits, and, when the record does not answer for `main`, how
long `main`'s head has waited for one. It exits 1 when one has waited longer than the roster's `stale_after_hours`
(24 by default) or could not be judged, so a timer or cron job on another machine with read access can mail on
failure:

```sh
sentinel4rust --roster roster.toml --status
```

It keeps its own clones and lock under `<work_dir>/status`, so it can also run beside the service.

## 6. Running cargo-mutants without the keys

Every repository's build scripts, tests and dependencies run where cargo-mutants runs. To keep them away from the
deploy keys, cargo-mutants can run in a unit of its own, `sentinel4rust-mutants@.service`, with
`InaccessiblePaths=/home/sentinel/.ssh`; git, which needs the keys, stays in the service. The service starts the unit
through a polkit rule that lets the `sentinel` user start that unit's instances and nothing else.

```sh
sudo cp deploy/sentinel4rust-mutants@.service /etc/systemd/system/
sudo cp deploy/50-sentinel4rust-mutants.rules /etc/polkit-1/rules.d/
sudo systemctl daemon-reload
sudo -iu sentinel /path/to/sentinel4rust/deploy/verify-isolation.sh /home/sentinel/sentinel4rust
```

Then add `run_unit = "sentinel4rust-mutants"` to the roster and restart the service. Each cargo-mutants command is
written to `<work_dir>/jobs/run/job.toml`, the unit runs it with `sentinel4rust --run-job` and writes the result and
output beside it, and the service reads them back. The unit's paths assume `work_dir = /home/sentinel/sentinel4rust`;
change `ExecStart=` if yours differs. This has not yet been proven on the machine: `verify-isolation.sh` is that
proof.
## 7. Deciding what a public repository publishes

On a public repository, `mutants` is public too: the survivor lists -- a map of the code no test pins -- and, on
failures, up to 200 lines of log tail, with paths such as `/home/sentinel/...`. Decide per repository, before adding
it to the roster, whether that map may be public; a repository whose survivors should not be is left off the roster,
and nothing about it is published. Record the decision beside its roster entry, or in the roster's comments when it
is left off.

Commits on `mutants` are unsigned, from `sentinel4rust@localhost`: a repository requiring signed commits on every
branch rejects them, so require them on `main` only (section 2's ruleset leaves `mutants` without rules).
