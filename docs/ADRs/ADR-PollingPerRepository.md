# ADR — The pause is a polling interval per repository

## Status

Accepted.

## Context

The loop visits a list of repositories forever. A fixed sleep after each visit would check one repository every five
minutes but twelve only once an hour; visiting in roster order alone would let a repository whose `main` keeps moving
starve the rest.

## Decision

`pause_seconds` is a polling interval per repository: no repository is visited twice within it, the loop sleeps only
when none is due, and among due repositories the one that has waited longest goes first, ties in roster order.

## Forcing constraints / Evidence

With one repository this is a plain "check every five minutes"; with twelve, each is still checked every five
minutes. One repository runs at a time, so a long run delays the others; the run cap bounds the delay
(`ADR-ResultsFailuresAndBackOff.md`). If `main` moves during a run, the run publishes the hash it tested and the next
visit picks up the new one.

## Rejected alternatives

- **A fixed sleep between visits.** The interval grows with the roster.
- **Parallel runs.** cargo-mutants already uses the machine's cores through `--jobs`; two runs would fight for them.
- **Webhooks.** Need a reachable endpoint for a machine under a desk; polling needs nothing.

## Consequences

The schedule and back-off are in memory: losing them on a restart costs at most an early visit. The loop never sleeps
less than a second, so a clock that does not move cannot spin it. `--once` visits every repository once, in roster
order, ignoring the schedule.

## Enforcement

`Schedule` and `RepoState` (`core/src/patrol/`) with `core/tests/patrol/schedule_tests.rs` and `repo_state_tests.rs`;
the sentinel's `step` tests on a fake clock (`core/tests/support/world.rs`).

## Related

- `ADR-ResultsFailuresAndBackOff.md`, `ADR-InjectedTime.md`.
