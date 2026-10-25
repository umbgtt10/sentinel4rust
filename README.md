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
summary.toml       commit, times, outcome, counts, new/gone, versions
missed.txt         surviving mutants, without line numbers, so `git log -p` is the history of the survivors
timeout.txt        mutants whose tests timed out (absent when none)
located/           the same lists with file:line:col
```

Commit messages read `Mutants of 1a2b3c4: 5 missed (+1 -2), 120 caught`.

A repository is tested only once it has a `.cargo/mutants.toml` on `main`; crate filters and accepted survivors
(`exclude_re`, `#[mutants::skip]`) live there.

## Running it

```sh
sentinel4rust --roster ~/.config/sentinel4rust/roster.toml          # forever
sentinel4rust --roster ~/.config/sentinel4rust/roster.toml --once   # one pass; exit 1 if a visit failed
```

One journal line per visit:

```
repo=twin4rust step=published hash=1a2b3c4 outcome=completed missed=5 caught=120 timeout=0 unviable=3 new=1 gone=2 duration_s=1834
repo=etheram-raft step=unchanged hash=9f8e7d6 duration_s=1
```

- Roster format: [`deploy/roster.example.toml`](deploy/roster.example.toml)
- Machine set-up, keys and rulesets: [`deploy/SETUP.md`](deploy/SETUP.md)
- Architecture: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md); decisions: [`docs/ADRs/`](docs/ADRs/README.md); the original
  design: [`docs/history/`](docs/history/README.md)

## Development

```sh
just stage1   # fmt, clippy (pedantic, -D warnings), unit + git + end-to-end tests with a stub cargo-mutants
just stage2   # stern4rust, cargo-shear, dry4rust, crap4rust, twin4rust, iceberg4rust, then the real cargo-mutants 27.1.0
```

The core has no dependencies. Layout: `core/` (library and binary), `xtask/` (the gates).

## License

MIT -- see [LICENSE](LICENSE).
