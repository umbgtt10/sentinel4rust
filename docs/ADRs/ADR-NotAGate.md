# ADR — Not a gate

## Status

Accepted.

## Context

Full mutation testing of a repository takes minutes for a small tool and hours for a large workspace. Run on every
push, it would make every push wait, or need machines nobody wants to pay for; run before a merge, it would turn a
question about the tests into a reason to block the code.

The question it answers is worth asking all the same: *which changes to this code would its tests not notice?* It is
a question about one commit, and its answer does not go stale until the code does.

## Decision

`sentinel4rust` answers that question after the fact, on a dedicated machine, for every commit `main` reaches, and
publishes the answer on the repository's own `mutants` branch; nothing waits for it, and it never touches `main`.

## Forcing constraints / Evidence

A personal tool: one user, one machine under the desk, nothing else running on it. The only measured run is
twin4rust's, 215 mutants in 3 min 20 s at `-j 1`; etheram-ibft's core packages alone have 2,357 mutants
(`docs/OPEN_POINTS.md`, point 3). A gate at that scale is not viable; a background answer is.

## Rejected alternatives

- **A CI job per push.** Hours per push on shared runners, and a red mark for a question that is not pass or fail.
- **A pre-merge gate.** Makes survivors a reason to stop work instead of a list to work through.
- **Running by hand.** The answer is wanted for every commit, and a hand run is skipped exactly when it matters.

## Consequences

Survivors are known minutes or hours after a commit lands, not before. The tool's silence is ambiguous: a machine
that stopped looks like a repository that did not move (`docs/OPEN_POINTS.md`, point 12). It exits non-zero only for
`--once` passes and start-up failures; a repository's survivors never fail anything.

## Enforcement

`GitCli` pushes `refs/heads/mutants` and nothing else (`ADR-ExactCommitOnlyMutantsPushed`), and the real-git tests in
`core/tests/system/git/git_cli_tests.rs` assert `main` is byte-identical after every publish.

## Related

- `ADR-StateLivesOnMutants.md`, `ADR-ExactCommitOnlyMutantsPushed.md`, `ADR-PollingPerRepository.md`.
