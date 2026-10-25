# Provisioning the machine -- for the agent

This is written for you: Claude Code on the sentinel's machine, started by the owner in their checkout of
sentinel4rust, as the owner's own sudo user. By the time you read it, the owner has set the BIOS, installed Ubuntu
Server with that user, run `gh auth login`, `gh repo clone umbgtt10/sentinel4rust` and `sudo -v`, started you in the
checkout and asked you to execute this file. When you are done the machine is provisioned, the repositories are
onboarded, the service runs unattended around the clock, and no GitHub credential of the owner's is left on it.

Four scripts do the work. Each is idempotent, has `--dry-run`, and stops at its first failure with a `FAIL` line.
[`SETUP.md`](SETUP.md) is the reference of what each step does by hand, and why.

| script | run as | does |
|---|---|---|
| `deploy/provision.sh` | `sudo` | packages, unattended upgrades, the `sentinel` user, Rust 1.99.1, cargo-mutants 27.1.0, sentinel4rust from HEAD, the roster skeleton, GitHub's host keys, the units, the polkit rule, the isolation proof |
| `deploy/onboard.sh` | the owner | per repository: the sentinel's key and host alias, the ruleset and the deploy key on GitHub, the access proof, the roster entry |
| `deploy/go-live.sh` | `sudo` | the first `--once` pass, then the service enabled and started |
| `deploy/status-check.sh` | anyone, elsewhere | `--status` from another machine, for cron or a timer |

## Rules

1. **Never print a private key or a token.** Never `cat` a `~sentinel/.ssh/sentinel_*` file without `.pub`, never
   run `gh auth token` or `gh auth status --show-token`, never show `~/.config/gh/`. Public keys and fingerprints are
   fine.
2. **GitHub changes only after the owner's explicit go in the chat.** Step 5 shows the plan -- per repository,
   exactly what will change on GitHub -- and asks; pass `--apply-github` to `onboard.sh` only after the owner has
   answered with a go naming the repositories. A go covers those repositories and that plan, nothing else: another
   repository, or a plan that changed, needs a new one. Nothing else in this file changes GitHub.
3. **Keep the checkout as it is.** Do not edit, commit, pull or push in it, and do not edit the scripts, the units or
   the rule to get past a failure. The machine runs exactly what the owner pushed.
4. **sudo is cached, not given.** Use `sudo -n` so that nothing waits for a password. When `sudo -n true` fails,
   stop and ask the owner to refresh it -- for example by leaving Claude Code, running `sudo -v` and starting you
   again in the checkout, after which you carry on from the step you were at. Never ask for a password in the chat.
5. **Stop and ask** when a check below does not hold, when a step fails a second time, when a script says `BLOCKED`,
   or when anything happens that this file does not cover. Say what you ran, what it printed, and what you propose.
6. **At the very end, `gh auth logout`** (step 9), and tell the owner you did.

Write each script's output to a log in your home rather than a pipe: `~/sentinel-<step>.log`, then read it. The long
steps -- provisioning, and above all going live -- may outlast your command timeout; run them in the background
where your tools allow, and read the log.

## 0. Preconditions -- check all of them first, change nothing

| check | command | must show |
|---|---|---|
| Ubuntu | `. /etc/os-release && echo "$ID $VERSION_ID"` | `ubuntu`, a release the owner chose (the scripts were checked on 26.04) |
| systemd | `test -d /run/systemd/system && systemctl --version` | a systemd version |
| polkit | `systemctl is-active polkit` | `active`, or not installed (step 2 installs `polkitd`); installed but not active: stop |
| sudo cached | `sudo -n true && echo cached` | `cached` |
| gh logged in | `gh auth status` | logged in to github.com as `umbgtt10`, with the `repo` scope |
| the checkout | `git status --porcelain` | nothing, Claude Code's own untracked `.claude/` aside |
| at a pushed commit | `git fetch --quiet origin && git merge-base --is-ancestor HEAD origin/main && echo pushed` | `pushed` |
| the network | `curl -sSfI https://sh.rustup.rs > /dev/null && echo up` | `up` |
| the machine | `nproc; free -g; df -h /home` | report them to the owner: they size `jobs` (SETUP.md section 4) |

Report the table to the owner in one message. Go on only if every row holds.

## 1. Provisioning plan

```sh
sudo -n deploy/provision.sh --dry-run > ~/sentinel-provision-plan.log 2>&1; echo "exit=$?"
```

It prints each action as `ok` (already in place) or `would` (to do), with every file it would write, or the change
to it. Exit 0. It changes nothing; provisioning the machine is what the owner asked for, so no go is needed. A
`FAIL` here (not Ubuntu, a host key GitHub does not publish) is a stop.

## 2. Provision

```sh
sudo -n deploy/provision.sh > ~/sentinel-provision.log 2>&1; echo "exit=$?"
```

Ten to thirty minutes: rustup, the toolchain, and cargo-mutants and sentinel4rust compiled. It ends with `== done`
and `The service is not enabled`. Then check:

```sh
sudo -n -u sentinel -H /home/sentinel/.cargo/bin/sentinel4rust --version     # sentinel4rust 0.1.0
sudo -n -u sentinel -H /home/sentinel/.cargo/bin/cargo mutants --version       # cargo-mutants 27.1.0
sudo -n cat /home/sentinel/src/sentinel4rust/INSTALLED_COMMIT                  # git rev-parse HEAD
sudo -n grep -n '^run_unit' /home/sentinel/.config/sentinel4rust/roster.toml   # run_unit = "sentinel4rust-mutants"
grep -n 'RESULT' ~/sentinel-provision.log                                      # RESULT: ok (the isolation proof)
systemctl is-enabled sentinel4rust                                             # disabled
timedatectl show -p NTPSynchronized --value                                    # yes
```

**Stop** when:

- it exits non-zero. Show the `FAIL` line and the twenty lines before it. A plain network failure (apt, rustup,
  crates.io) may be re-run once; anything else, or a second failure, is a stop.
- the isolation proof fails. Show its lines verbatim: it is the proof `docs/OPEN_POINTS.md` point 5 waits for, and
  `run_unit` stays unset without it.
- the clock is still not synchronised a few minutes later.

## 3. The repositories

Unless the owner's request named them, ask which repositories to onboard. For each one, the owner decides, and you
tell them, before step 5:

- **whether its `mutants` branch may be public** -- on a public repository it is: the surviving mutants, a map of the
  code no test pins, and on failures a log tail with paths (SETUP.md section 7). A repository whose survivors should
  not be public stays off the roster.
- whether it has a `.cargo/mutants.toml` on `main`: without one the sentinel skips it, harmlessly, until it has one.
  The dry run in step 4 says.
- for a large repository, whether to measure one run first (SETUP.md section 4) to size `jobs` and
  `run_timeout_hours`.

## 4. Onboarding plan

```sh
deploy/onboard.sh --dry-run <repo>... > ~/sentinel-onboard-plan.log 2>&1; echo "exit=$?"
```

It reads GitHub and writes nothing, anywhere. Per repository it says whether it is public or private, whether
`.cargo/mutants.toml` is there, and, as `ok` or `would`, each of: the ruleset, the sentinel's key and host alias, the
deploy key, the access proof, the roster entry. A repository marked `BLOCKED` is one on which GitHub offers no
ruleset -- a private repository on the free plan: nothing would keep its write key off `main`. Leave it out of
step 5 and put the owner's choices to them: make it public, move to a paid plan, or leave it off the roster
(`docs/ADRs/ADR-DeployKeyAndRuleset.md`).

## 5. The owner's go

Show the owner, in the chat, exactly what will change, per repository, from the plan:

```text
umbgtt10/<repo> (PUBLIC|PRIVATE), mutants branch public: yes|no
  GitHub:
  - ruleset "sentinel: only mutants" created, from deploy/rulesets/sentinel-only-mutants.json: on every branch
    but mutants, creating, updating, deleting and force-pushing restricted to repository admins   | already there
  - deploy key "sentinel4rust" added, WITH WRITE ACCESS: the sentinel's key sentinel_<repo>      | already there
  - a probe push of a branch sentinel4rust-probe, which the ruleset must reject
  This machine:
  - key ~sentinel/.ssh/sentinel_<repo>, host alias github-<repo>, the [[repo]] entry in the roster
```

and tell them one consequence once: gh adds a deploy key under its own authorization, so **revoking the GitHub CLI's
access in GitHub's settings (Applications) deletes these keys** -- `gh repo deploy-key add --help` says so. `gh auth
logout` in step 9 revokes nothing and does not touch them.

Then ask for a go and wait. Do nothing on GitHub until it comes. A go for some of the repositories covers those.

## 6. Onboard

```sh
deploy/onboard.sh --apply-github <the repositories of the go> > ~/sentinel-onboard.log 2>&1; echo "exit=$?"
```

Exit 0; for each repository `RESULT: ok` from the access proof and an `append [[repo]]` or an `ok ... on the
roster`. Check the roster -- it holds no secret:

```sh
sudo -n -u sentinel -H cat /home/sentinel/.config/sentinel4rust/roster.toml
```

**Stop** when the access proof fails (`creating a branch was ACCEPTED`: the ruleset is not in force), when a deploy
key titled `sentinel4rust` is there that is not this machine's (a previous machine's, perhaps: the owner deletes it),
or on anything `BLOCKED`. Do not change anything on GitHub by other means to get past it.

## 7. Go live

```sh
sudo -n deploy/go-live.sh > ~/sentinel-go-live.log 2>&1; echo "exit=$?"
```

With the service stopped it runs one `sentinel4rust --once` pass as `sentinel` -- one full cargo-mutants run per
repository that has no current record: minutes for a small repository (iceberg4rust, 325 mutants: four minutes with
`jobs = 2` on the development laptop), many hours for a large one. The pass runs in the transient unit
`sentinel4rust-first-pass`, which outlives the script: if your command is cut off, the pass goes on. Follow it with
`journalctl -u sentinel4rust-first-pass -n 20 --no-pager`, never stop it unless the owner asks, and run the same
command again -- it waits for the pass, makes a quick second one and decides on that.

It shows the pass's journal lines: one `repo=<name> step=...` per repository. `published` and `unchanged` are good.
`skipped reason=no-filters` is a repository without `.cargo/mutants.toml`, and `skipped reason=toolchain-missing` one
whose `rust-toolchain.toml` names a compiler the machine lacks: report both, neither is a failure. Then it enables
and starts the service and checks that it stays up: `ok sentinel4rust is enabled and active`. Confirm:

```sh
systemctl is-enabled sentinel4rust; systemctl is-active sentinel4rust   # enabled, active
journalctl -u sentinel4rust -n 10 --no-pager                            # the start-up line, then visits
```

**Stop** when it exits 1. A `backed-off`, `unreachable` or `unreadable` line is a failed visit, and the service stays
off: show the lines and ask. Do not enable the service by hand.

## 8. Check the records

```sh
sudo -n -u sentinel -H /home/sentinel/.cargo/bin/sentinel4rust --roster /home/sentinel/.config/sentinel4rust/roster.toml --status
```

Read-only, beside the running service: one line per repository, `state=current` for each the pass published,
`skipped` for the skipped ones; exit 0.

## 9. Log out of GitHub

```sh
gh auth logout --hostname github.com --user umbgtt10
gh auth status    # You are not logged into any GitHub hosts
```

The sentinel needs none of it: it reaches GitHub only with its deploy keys. Tell the owner the server no longer holds
their GitHub credential, that the logout only removed it locally, and that a later onboarding needs `gh auth login`
again.

## 10. Report

To the owner: the commit installed (`INSTALLED_COMMIT`), each repository's first journal line, what was skipped and
why, every `WARN` the scripts printed, the machine's size against `jobs`, that `gh` is logged out, and how to watch
it from another machine (below).

## When a step fails

Every script prints `FAIL` with the reason, or `stopped by the failing command above` after the command's own
message. Read the log from that line back. A transient network failure may be re-run once; every other failure is a
stop (rule 5). A refusal is never worked around: a host key GitHub does not publish, `BLOCKED`, a failed access
proof, a failed isolation proof.

## Re-running

Everything is idempotent. `provision.sh` leaves what is in place, and on a HEAD it has not built yet it rebuilds
sentinel4rust from it -- that is how an upgrade goes, when the owner asks for one: `git pull --ff-only` in the
checkout (after `gh auth login`: the checkout is fetched with the owner's credential), `sudo deploy/provision.sh`,
then the restart below. `onboard.sh` leaves keys, host aliases, rulesets, deploy keys and entries that are there. `go-live.sh` only
says so once the service is live.

## Restarting the service

The roster and the binary are read on start, so a new roster entry or an upgrade needs a restart. Restart only
while no run is in progress -- a run in the run unit outlives a restart of the service (`docs/OPEN_POINTS.md`
point 19):

```sh
systemctl is-active sentinel4rust-mutants@run.service    # inactive: no run in progress
sudo systemctl restart sentinel4rust
journalctl -u sentinel4rust -n 3 --no-pager               # the start-up line: N repositories
```

## Onboarding one more repository later

The owner runs `gh auth login` and `sudo -v` again, starts you in the checkout and asks you to onboard
`<repo>` as this file says. Then: step 0's checks, steps 3 to 6 for that repository, the restart above, the
`--status` of step 8, and step 9.

## Reading the service's state

`journalctl` reads the system journal for members of the `adm` group, which the user the installer created is in;
otherwise prefix it with `sudo -n`.

| what | command |
|---|---|
| is it running | `systemctl status sentinel4rust --no-pager` |
| what it did | `journalctl -u sentinel4rust -n 50 --no-pager`, or `-f`; one line per visit: `unchanged`, `published`, `backed-off`, `unreachable`, `skipped`, `unreadable` |
| the run unit | `journalctl -u sentinel4rust-mutants@run.service -n 20 --no-pager` (cargo-mutants' own output stays in `<work_dir>/jobs/run/`) |
| how far behind | `sudo -u sentinel -H /home/sentinel/.cargo/bin/sentinel4rust --roster /home/sentinel/.config/sentinel4rust/roster.toml --status` (exit 1: one needs attention) |
| upgrades, reboots | `systemctl list-timers 'apt-daily*'`, `ls /var/run/reboot-required`, `last -n 5 reboot` |
| the results | `git log -p origin/mutants` in any clone of a repository |

## Watching it from another machine

A dead sentinel looks like quiet repositories, so the owner's laptop, say, runs `--status` on a timer through
`deploy/status-check.sh`, with its own access -- https, or the owner's SSH key -- never the sentinel's keys. On that
machine, from a checkout -- at `~/sentinel4rust` below --: `cargo install --path core --locked`, then a roster of
its own, e.g. `~/.config/sentinel4rust/status.toml`:

```toml
work_dir = "/home/<you>/.cache/sentinel4rust-status"   # its clones, under <work_dir>/status
stale_after_hours = 24                                  # as on the machine

[[repo]]
name = "twin4rust"
url = "https://github.com/umbgtt10/twin4rust.git"      # a private one: git@github.com:umbgtt10/<repo>.git
```

`status-check.sh` prints nothing while every record is current or has waited less than `stale_after_hours`;
otherwise it prints the lines, mails them with `--mail <address>` when `mail` (mailutils or bsd-mailx) is there,
and exits 1. From cron, which mails what a job prints:

```sh
MAILTO=you@example.com
0 */6 * * * $HOME/sentinel4rust/deploy/status-check.sh --roster $HOME/.config/sentinel4rust/status.toml --sentinel $HOME/.cargo/bin/sentinel4rust
```

Or a systemd user timer, `~/.config/systemd/user/sentinel4rust-status.service` and `.timer`:

```ini
[Unit]
Description=sentinel4rust --status, from this machine

[Service]
Type=oneshot
ExecStart=%h/sentinel4rust/deploy/status-check.sh --roster %h/.config/sentinel4rust/status.toml --sentinel %h/.cargo/bin/sentinel4rust --mail you@example.com
```

```ini
[Unit]
Description=sentinel4rust --status every six hours

[Timer]
OnCalendar=*-*-* 00/6:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

`systemctl --user daemon-reload && systemctl --user enable --now sentinel4rust-status.timer`; a failed check shows
in `systemctl --user --failed` and `journalctl --user -u sentinel4rust-status`. A user timer runs only while the user
is logged in, unless `loginctl enable-linger`.
