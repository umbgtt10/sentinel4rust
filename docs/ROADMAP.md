# Roadmap

What has shipped, what is under way, and what is under consideration.

## Shipped

### The first implementation

The service as [IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md) describes it, in one commit (`48c8fb1`), built from
the design now in [history/](history/README.md).

### Brought into the family

The implementation arrived without the gates and coding standards the other `*4rust` tools follow (open point 7,
now closed). It was brought under them in three phases, in this order, one revertable commit per slice, each
test-first and green on both stages before it landed. Phases 1 and 2 are done; phase 3 is under way below.

#### 1. Gates first, with baselines

| slice | what |
|---|---|
| G0 | this plan |
| G1 | the compiler pinned at 1.99.1, the `cargo xtask` alias (open point 6), the copyright header on every `.rs` file, `stern4rust.toml`, a `justfile` with `stage1` and `stage2` |
| G2 | the xtask rebuilt from `Gate` implementations against a `CommandRunner`, with tests of its own; CI running the same two commands; stern4rust, cargo-shear and dry4rust switched on against baselines of today's offences |
| pin | (landed after R6) the stern gate runs stern4rust pinned at commit `8b0e8e5` of its repository, installed under `target/stern4rust`; its baseline re-recorded with that binary |

The stern gate runs stern4rust pinned at one pushed commit, the full rule set of its next release, installed under
`target/stern4rust` rather than over the global `cargo-stern4rust`; when that release is out, the pin moves to its
version. The baseline is recorded with the pinned binary. It only ever shrinks: each later slice may remove entries and
never adds one.

#### 2. Restructure to the family layout

Bottom-up, one module per slice. A slice moves a module into the layout below, one object per file, its
`#[cfg(test)]` tests into mirrored files under `core/tests/` (Arrange/Act/Assert, named
`<method>_<conditions>_<result>`), and fixes every offence of the files it moves, so a moved file never needs a new
baseline entry. No test is deleted; every assertion moves with its test.

| slice | what | gates switched on |
|---|---|---|
| R1 | the stray root `src/main.rs` (the 0.0.1 placeholder) deleted | |
| R2 | `core/tests/` as one registry: `all_tests.rs`, `support/`, the end-to-end and git tests renamed after their sources | |
| R3 | the TOML subset → `model/toml/` | crap4rust, iceberg4rust |
| R4 | the model's values; the re-exports in `model/mod.rs` go | |
| R5 | `outcome.rs` split, one type per file | |
| R6 | what the `mutants` branch holds → `model/branch/` | |
| R7 | the roster → `model/onboarding/` | |
| R8 | one trait per file in `traits/`; the contracts' data in `model/`; typed errors | |
| R9 | processes, clock, pause, journal → `system/process/` | |
| R10 | git → `system/git/` | |
| R11 | cargo-mutants → `system/mutants/` | |
| R12 | the schedule → `patrol/` | |
| R13 | the sentinel → `patrol/` | |
| R14 | the command line and the composition root → `invocation/`; the stern baseline is empty and deleted | twin4rust: all six on |
| R15 | `docs/DESIGN.md` verbatim to `docs/history/`; `ARCHITECTURE.md`; the design's decisions as ADRs | |
| R16 | ROADMAP, IMPLEMENTED-FEATURES, CHANGELOG, CLAUDE.md and README in the siblings' shape | |

The layout it ends in:

```
core/src/
  invocation/   the command line, the lock, EntryPoint (the composition root)
  traits/       one contract per file
  model/        the data the contracts carry; branch/ (summary.toml, the published files),
                onboarding/ (the roster), toml/ (the dependency-free TOML subset)
  patrol/       the loop: Sentinel, Schedule, back-off, the journal line
  system/       the real implementations: process/, git/, mutants/
```

stern4rust allows a directory five subfolders, so `process/`, `git/` and `mutants/` sit under `system/`.

## Under way: the open points, one test-first slice each

In this order: 2 (install with `--path`), 14 (`--force-with-lease`), 1 (`TMPDIR` under `work_dir`), 4 (an unreadable
`summary.toml` is its own state), 13 (the cargo-mutants version counts toward current), 3 (per-repository run caps, a
measuring procedure), 9 (real cargo-mutants fixtures for exit 4 and exit 3), 8 (the docs agree with the code), 10 (in
three commits: submodules, path dependencies outside the repository, the toolchain), 11 (new survivors re-checked
once), 12 (a staleness check), 5 (cargo-mutants in a separate systemd unit without `~/.ssh`), 15 (what each public
repository publishes), 16 (the smaller things). Each closes in `docs/OPEN_POINTS.md` as it lands.

## Under consideration

The ideas of the design's section 8 that no open point takes up, each only if real use asks for it:

- **Incremental runs** (`--in-diff` against the last full record, with a periodic full run), if full runs take too
  long. `kind = "full"` is already in `summary.toml`.
- **Docs-only commits** republishing the previous lists instead of a full run.
- **`--in-place`** with a warm `target/` instead of `--jobs` copies, if measurement favours it.
- **Reloading the roster** without a restart.
- **A notification** when survivors rise, if the commit messages on `mutants` turn out not to be enough.

Two of its ideas became open points: re-checking new survivors once before publishing (11, done), and, instead of a
heartbeat, a check that compares each repository's `main` with its `mutants` record (12).
