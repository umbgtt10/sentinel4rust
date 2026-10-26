# ADR — Time is injected

## Status

Accepted.

## Context

The loop's behaviour is about time: polling intervals, back-off doubling to a six-hour cap, deadlines on every
command. Tested against the real clock, each of those tests would sleep for real or be skipped.

## Decision

Every clock read and every wait goes through a trait -- `Clock` and `Pause` -- and deadlines are enforced inside the
`CommandRunner` implementation, so nothing outside those adapters reads the clock.

## Forcing constraints / Evidence

stern4rust's `injected-time` rule forbids reading a clock or waiting on a timer in library code outside a port's
implementation, and `timeless-tests` forbids it in tests. Both run in stage 2.

## Rejected alternatives

- **`Instant::now()` where needed.** Untestable without sleeping, and forbidden by the rules above.
- **A global fake clock.** Shared state between tests that run in parallel.

## Consequences

The schedule, back-off and the whole loop are tested on a fake clock in `World`, without sleeping. The adapters
themselves (`SystemClock`, `ThreadPause`, `SystemCommandRunner`) are tested against the real system: the pause test
costs one second, and the deadline test asserts on captured output rather than on elapsed time.

## Enforcement

The stern gate (`injected-time`, `timeless-tests`); `core/tests/support/world.rs`; the tests in
`cli/tests/system/process/`.

## Related

- `ADR-PollingPerRepository.md`, `ADR-ResultsFailuresAndBackOff.md`.
