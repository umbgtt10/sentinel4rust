# sentinel4rust

Sentinels hunt mutants.

`sentinel4rust` runs on a dedicated machine, around the clock: it walks a list of repositories, and whenever a
repository's `main` moves it runs full mutation testing with [cargo-mutants](https://mutants.rs) against that exact
commit, then publishes the surviving mutants, with the commit hash, on the repository's own `mutants` branch. It is
not a gate: nothing waits for it, and it never touches `main`.

## What a repository gets

An orphan `mutants` branch, one commit per tested `main` commit:

```
README.md          what the branch is; do not merge
summary.toml       commit, times, outcome, counts, new/gone, versions, the test timeout
missed.txt         surviving mutants, without line numbers, so `git log -p` is the history of the survivors
timeout.txt        mutants whose tests timed out (absent when none)
located/           the same lists with file:line:col
```

Commit messages read `Mutants of 1a2b3c4: 5 missed (+1 -2), 120 caught`, and end `, 3 timed out` when any did.

A repository is tested only once it has a `.cargo/mutants.toml` on `main`; crate filters and accepted survivors
(`exclude_re`, `#[mutants::skip]`) live there.

## Running it

```sh
sentinel4rust --roster ~/.config/sentinel4rust/roster.toml          # forever
sentinel4rust --roster ~/.config/sentinel4rust/roster.toml --once   # one pass; exit 1 if a visit failed
sentinel4rust --roster ~/.config/sentinel4rust/roster.toml --status # fetch only: how far each record is behind main
```

One journal line per visit:

```
repo=twin4rust step=published hash=1a2b3c4 outcome=completed missed=5 caught=120 timeout=0 unviable=3 new=1 gone=2 duration_s=1834
repo=etheram-raft step=unchanged hash=9f8e7d6 duration_s=1
```

A published run whose timeouts outnumber its caught mutants adds `warning=timeouts-outnumber-caught` before
`duration_s`, and `--status` adds it to the repository's line and exits 1: its lists say little, and the likeliest
cause is a test timeout below the tests' own run ([`docs/OPEN_POINTS.md`](docs/OPEN_POINTS.md) point 10).

- Roster format: [`deploy/roster.example.toml`](deploy/roster.example.toml)
- Machine set-up, automated: [`deploy/PROVISION.md`](deploy/PROVISION.md), which an agent on the machine executes:
  `deploy/provision.sh`, `deploy/onboard.sh`, `deploy/go-live.sh`, and `deploy/status-check.sh` to watch it from
  elsewhere
- What each step does by hand, keys and rulesets: [`deploy/SETUP.md`](deploy/SETUP.md)

## Documentation

| Doc | What's in it |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | How a run flows through the code, module by module; the roster's keys. |
| [`docs/ADRs/`](docs/ADRs/README.md) | Why the tool is shaped the way it is. |
| [`docs/IMPLEMENTED-FEATURES.md`](docs/IMPLEMENTED-FEATURES.md) | What it does today. |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | What's shipped, what's under way, what's under consideration. |
| [`docs/OPEN_POINTS.md`](docs/OPEN_POINTS.md) | Known gaps, deliberately recorded. |
| [`docs/history/`](docs/history/README.md) | The original design, unchanged. |
| [`CHANGELOG.md`](CHANGELOG.md) | What changed. |

## Development

```sh
just stage1   # fmt, clippy (pedantic, -D warnings), the tests: unit, git and end to end with a stub cargo-mutants
just stage2   # the technology-free core, stern4rust, cargo-shear, dry4rust, crap4rust, twin4rust, iceberg4rust,
              # then the real cargo-mutants 27.1.0
```

Layout: `core/` (`sentinel4rust-core`: the logic, `no_std`, with no dependency and no technology), `cli/`
(`sentinel4rust`: the binary and every implementation, installed with `cargo install --path cli --locked`, or the
release with `cargo install sentinel4rust --locked`), `xtask/`
(the gates, with tests of their own). cargo-mutants mutates the core only (`.cargo/mutants.toml`).
CI (`.github/workflows/ci.yml`) runs both stages on Linux. The ruleset for `main` -- a pull request with both jobs
green -- is kept in [`.github/rulesets/main.json`](.github/rulesets/main.json), as GitHub holds it server-side.

Contributions: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Support: [`DONATE.md`](DONATE.md).

## License

MIT -- see [LICENSE](LICENSE).
