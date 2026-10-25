# CLAUDE.md

Notes for agents working in this repository.

- Read `docs/DESIGN.md` first; section 9 records where the code differs from the design.
- `just stage1` and `just stage2` must pass before every commit: stage 1 is fmt, clippy pedantic with `-D warnings` and
  all tests; stage 2 the end-to-end test against the real cargo-mutants.
- No dependencies: keep it that way unless a dependency removes real code. TOML goes through `core/src/toml.rs`.
- Every external effect goes through a trait in `core/src/traits.rs`; `Sentinel` gets them all through
  `Dependencies`. New behaviour in the loop is tested with the fakes in `core/src/sentinel/tests.rs`.
- `GitCli` must only ever push `refs/heads/mutants`. Tests assert `main` is byte-identical after every publish.
- Git tests use bare repositories and `file://` remotes under the temp dir (`core/tests/support`); no network.
