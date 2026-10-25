# CLAUDE.md

Notes for agents working in this repository.

- Read `docs/DESIGN.md` first; section 9 records where the code differs from the design.
- `just stage1` and `just stage2` must pass before every commit. Stage 1 is fmt, clippy pedantic with `-D warnings`
  and all tests. Stage 2 is `cargo xtask stage2`, the gates in this order: stern4rust (house rules, both packages,
  pinned), `cargo shear` (dependencies), `cargo dry4rust` (duplication in `core/src`, floor 25), `cargo crap4rust`
  (threshold 15), `cargo iceberg4rust` (ceiling 20), then the end-to-end tests against the real cargo-mutants 27.1.0
  (the `#[ignore]`d ones). twin4rust is built into the xtask and switched on when `docs/ROADMAP.md`'s restructure
  makes it pass.
- The stern gate runs stern4rust pinned at commit `8b0e8e5` of `https://github.com/umbgtt10/stern4rust` (a constant
  in `xtask/src/gates/stern_gate.rs`), installed repo-locally under `target/stern4rust`, never over the global
  `cargo-stern4rust`: the gate installs it whenever `target/stern4rust/bin/cargo-stern4rust` is missing or
  `target/stern4rust/REV` does not name the pin, locally and on CI alike.
- `stern4rust-baseline.json` and `dry4rust-baseline.json` record the offences and duplication there when the gates
  arrived. They only shrink: a change may remove entries, never add one. The stern baseline is recorded with the
  pinned binary; re-record it only to drop entries that are gone, then check no key was added:

  ```sh
  cp stern4rust-baseline.json /tmp/b-old.json
  target/stern4rust/bin/cargo-stern4rust stern4rust --manifest-path Cargo.toml --package sentinel4rust \
     --package xtask --write-baseline --baseline "$PWD/stern4rust-baseline.json"
  jq -e -s '((.[0].offences|keys) - (.[1].offences|keys)) == []' stern4rust-baseline.json /tmp/b-old.json
  cargo dry4rust --path core/src --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline
  ```
- The core has no dependencies: keep it that way unless a dependency removes real code. TOML goes through
  `core/src/toml.rs`. The dev-only `xtask` uses `serde` and `serde_json`, from `[workspace.dependencies]`.
- Every external effect goes through a trait in `core/src/traits.rs`; `Sentinel` gets them all through
  `Dependencies`. New behaviour in the loop is tested with the fakes in `core/src/sentinel/tests.rs`.
- `GitCli` must only ever push `refs/heads/mutants`. Tests assert `main` is byte-identical after every publish.
- Git tests use bare repositories and `file://` remotes under the temp dir (`core/tests/support`); no network.
