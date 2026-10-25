# ADR — Reading results, failures and back-off

## Status

Accepted.

## Context

A run can complete, find that the tests fail without any mutation, hit the cap, or fail for reasons that have
nothing to do with the code. Each needs a different answer, and none may leave the loop retrying one repository
forever.

## Decision

cargo-mutants' exit code decides the outcome, `mutants.out` the contents, and failures back off with a cap.

| result | handling |
|---|---|
| exit 0, 2, 3 | publish `completed` with the lists (3, timeouts, takes precedence over 2, so the counts come from the lists) |
| exit 4 | rerun once at once (flaky tests); a second 4 publishes `baseline-failed` with the log tail, no lists |
| any other exit, a signal, cannot start, run cap hit | publish nothing and back off; the 3rd tool failure on one commit publishes `tool-failed` with the reason and log tail |

## Forcing constraints / Evidence

The table matches cargo-mutants 27.1.0's exit codes (`0` all caught, `2` missed, `3` timeouts, `4` baseline failed,
`1`/`5`/`6`/`70` usage, diff filter, internal error), read in its `exit_code.rs`; cargo-mutants is pinned to that
version. `baseline-failed` and `tool-failed` record the commit, so they wait for `main` to move rather than rerun.

## Rejected alternatives

- **Retrying the baseline in `CargoMutantsCli`.** The runner reports one run; the retry is policy, and lives with
  the rest of the policy in `Sentinel`.
- **Publishing every tool failure.** A network blip or a full disk would overwrite a good record.
- **Unbounded retries.** A repository that cannot be run would hold the loop forever.

## Consequences

- **Back-off:** after a failure the next attempt waits `pause_seconds × 2^(failures-1)`, capped at 6 hours; any
  success, unchanged `main` or skip resets it. The tool-failure count is per commit: a new commit starts it again.
- **Unreachable:** git saying the repository is gone or refuses the key backs off straight to 6 hours and is logged
  once per change of state; it recovers on its own. The messages are matched in English, with `LC_ALL=C`.
- **Unreadable:** a `summary.toml` that does not parse is treated the same way, and the commit is never run over it
  (`ADR-StateLivesOnMutants.md`).
- **Timeouts:** every command has a deadline and is killed with its process group: git talking to the remote and
  creating a checkout 5 minutes, other git 1 minute, the cargo-mutants run `run_timeout_hours` (12 by default).
- **Log tails** keep the last 200 lines and at most 16 KiB of a failing run's output; nothing is redacted.
- **A missing toolchain** -- rustc does not run in the checkout -- is a skip, `toolchain-missing` with rustup's
  message, not a tool failure: cargo-mutants is not started and nothing backs off.

## Enforcement

`OutcomeInterpreter` (`core/src/system/mutants/outcome_interpreter.rs`) and its exit-code-table test;
`RepoState`'s back-off tests; the sentinel tests for a flaky baseline, a second baseline failure, the third tool
failure, a new commit, unreachable logged once, and transient failures; `UnreachableClassifier`'s tests;
`SystemCommandRunner`'s deadline test. Stage 2 runs the real cargo-mutants 27.1.0 on fixture crates for exit 2, 4
and 3, and stage 1 reads the lists two of those runs wrote (`core/tests/fixtures/mutants_out/`).

## Related

- `ADR-PositionFreeMutantKeys.md`, `ADR-PollingPerRepository.md`, `ADR-InjectedTime.md`.
