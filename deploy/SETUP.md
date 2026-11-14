# Setting up the machine

In order. Why it is set up this way: `docs/ADRs/ADR-SharedResultsRepository.md`.

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

## 2. The results repository and the watched repositories

The sentinel writes one repository, the results repository (`umbgtt10/mutants`), and only reads the watched ones.
The results repository is created once, private, by the owner:

```sh
gh repo create umbgtt10/mutants --private
```

On the machine, as `sentinel`, its one key and host alias:

```sh
ssh-keygen -t ed25519 -N "" -C "sentinel4rust mutants" -f ~/.ssh/sentinel_mutants
chmod 700 ~/.ssh && chmod 600 ~/.ssh/sentinel_*
cat >> ~/.ssh/config <<EOF
Host github-mutants
    HostName github.com
    User git
    IdentityFile ~/.ssh/sentinel_mutants
    IdentitiesOnly yes
    StrictHostKeyChecking yes
EOF
```

On GitHub, on the results repository only: Settings → Deploy keys → Add: `~/.ssh/sentinel_mutants.pub`, **Allow
write access**, titled `sentinel4rust`. A write deploy key can write every ref of its repository, and rulesets do not
hold it (`docs/ADRs/ADR-SharedResultsRepository.md`): that is why it goes on a repository that holds nothing but
results. GitHub accepts a deploy key on one repository only.

In the roster, before the first `[[repo]]`:

```toml
results_url = "git@github-mutants:umbgtt10/mutants.git"
```

A public watched repository needs nothing on GitHub: it is read over https with no credential --
`url = "https://github.com/umbgtt10/<repo>.git"` in its `[[repo]]` -- and must carry no deploy key of the sentinel's:
a `sentinel4rust` key left from an earlier design is deleted.

A private watched repository is read with a key of its own, `~/.ssh/sentinel_<repo>`, through a host alias
`github-<repo>` like the results repository's, added to it on GitHub as a **read-only** deploy key titled
`sentinel4rust` -- a read-only key cannot push at all -- and `url = "git@github-<repo>:umbgtt10/<repo>.git"`. Its
results go to the results repository like any other's, so it is watched only while the results repository is private.
When another watched repository depends on it as a git dependency (`etheram-core = { git =
"https://github.com/umbgtt10/etheram-core" }`), the build in the run unit, which holds no key, fetches it from the
sentinel's clone instead: in `~sentinel/.gitconfig`,

```ini
[url "file:///home/sentinel/sentinel4rust/repos/etheram-core.git"]
    insteadOf = https://github.com/umbgtt10/etheram-core
```

and, in `~sentinel/.cargo/config.toml`, `[net] git-fetch-with-cli = true`, so cargo fetches with git, which follows
the redirect. The clone keeps the branch at `refs/heads/<branch>` for this; the dependency's locked commit must be on
that branch, and the dependency must itself be on the roster, earlier than its dependants. `insteadOf` matches a
prefix: the redirect for `etheram-core` would catch an `etheram-core-x` too.

Every watched repository needs a `.cargo/mutants.toml` on `main` (it is skipped without one).

Verify, then add the repositories to the roster:

```sh
deploy/verify-access.sh git@github-mutants:umbgtt10/mutants.git https://github.com/umbgtt10/<repo>.git ...
```

It reads each watched repository's `main` -- a public one with no credential, a private one through its alias -- and
proves the key writes the results repository by
creating a branch, `sentinel4rust-probe`, and deleting it again; the results repository's `main` is never touched.
`deploy/onboard.sh <repo>...` does this section: the key, the host alias, the deploy key, the access proof,
`results_url`, the redirects and the roster entries. It changes GitHub -- the deploy key on the results repository
and a read-only one per private watched repository -- only with `--apply-github`, and refuses a private watched
repository while the results repository is public.

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
deploy key, cargo-mutants can run in a unit of its own, `sentinel4rust-mutants@.service`, with
`InaccessiblePaths=/home/sentinel/.ssh`; git, which needs the key, stays in the service. The service starts the unit,
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

## 7. Deciding what a repository publishes

The results repository is private, so publishing exposes nothing: every watched repository's survivor lists -- a
map of the code no test pins -- and, on failures, up to 200 lines of log tail, with paths such as
`/home/sentinel/...`, are readable by the owner and by the sentinel's key only. It was public until 2026-11-14 and
was made private that day; nothing in the tool depends on its visibility, so it stays a decision of the owner's. Were
it ever public again, decide per repository, before adding it to the roster, whether its survivors may be public.

Commits on the results repository are unsigned, from `sentinel4rust@localhost`: it must not require signed commits.
Nothing is ever written to a watched repository, so its own rules are its owner's business.
