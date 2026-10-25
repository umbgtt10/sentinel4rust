# CLAUDE.md

Notes for agents working in this repository.

- Read `docs/DESIGN.md` first; section 9 records where the code differs from the design.
- `just stage1` and `just stage2` must pass before every commit. Stage 1 is fmt, clippy pedantic with `-D warnings`
  and all tests. Stage 2 is `cargo xtask stage2`, the gates in this order: stern4rust (house rules, both packages,
  pinned), `cargo shear` (dependencies), `cargo dry4rust` (duplication in `core/src`, floor 25), `cargo crap4rust`
  (threshold 15), `cargo twin4rust` (every source file has its mirrored test file), `cargo iceberg4rust` (ceiling 7,
  a ratchet lowered as the worst file improves and never raised), then the end-to-end tests against the real
  cargo-mutants 27.1.0 (the `#[ignore]`d ones).
- The stern gate runs stern4rust pinned at commit `8b0e8e5` of `https://github.com/umbgtt10/stern4rust` (a constant
  in `xtask/src/gates/stern_gate.rs`), installed repo-locally under `target/stern4rust`, never over the global
  `cargo-stern4rust`: the gate installs it whenever `target/stern4rust/bin/cargo-stern4rust` is missing or
  `target/stern4rust/REV` does not name the pin, locally and on CI alike.
- There is no stern baseline: every rule holds, so a change that breaks one fails stage 2. `dry4rust-baseline.json`
  records no group; it may never gain one. Re-record it only to drop a group that is gone, at the same floor:
  `cargo dry4rust --path core/src --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline`.
- The core has no dependencies: keep it that way unless a dependency removes real code. TOML goes through
  `core/src/model/toml/`. The dev-only `xtask` uses `serde` and `serde_json`, from `[workspace.dependencies]`.
- Every external effect goes through a trait in `core/src/traits/`; `Sentinel` gets them all through
  `Dependencies`, and `EntryPoint::compose` builds the real ones. New behaviour in the loop is tested with the fake
  in `core/tests/support/world.rs`.
- `GitCli` must only ever push `refs/heads/mutants`. Tests assert `main` is byte-identical after every publish.
- Git tests use bare repositories and `file://` remotes under the temp dir (`core/tests/support`); no network.
