# Architecture Decision Records

Each file records one decision that shaped this codebase, with the forces that made it and the alternatives it
displaced. They are not tiered: `sentinel4rust` is one service with a small enough decision surface that a flat list is
sufficient. Most began as sections of the original design, kept unchanged in `docs/history/[SUPERSEDED] DESIGN.md`.

## Index

| ADR | Decision |
|---|---|
| [ADR-NotAGate](ADR-NotAGate.md) | It answers which changes the tests would not notice after the fact, for every commit `main` reaches; nothing waits for it and it never touches `main`. |
| [ADR-StateLivesOnMutants](ADR-StateLivesOnMutants.md) | The record on `mutants` is the only state: current when it names `head` and the roster's `config_version`. |
| [ADR-OrphanMutantsBranch](ADR-OrphanMutantsBranch.md) | Results live on an orphan branch, one overwriting commit per publication, so they cannot be merged and their log is the survivors' history. |
| [ADR-PositionFreeMutantKeys](ADR-PositionFreeMutantKeys.md) | A mutant is keyed by cargo-mutants' line without `:line:col`, with ordinals for duplicates; keys and counts come from the `.txt` lists. |
| [ADR-ExactCommitOnlyMutantsPushed](ADR-ExactCommitOnlyMutantsPushed.md) | Runs use a detached worktree at `head`; publishing uses plumbing alone and pushes `<commit>:refs/heads/mutants` and nothing else. |
| [ADR-FiltersLiveInEachRepository](ADR-FiltersLiveInEachRepository.md) | What is mutated and which survivors are accepted is said in the repository's `.cargo/mutants.toml`; without one it is skipped. |
| [ADR-PollingPerRepository](ADR-PollingPerRepository.md) | The pause is a polling interval per repository; the longest-waiting due repository goes first; one runs at a time. |
| [ADR-ResultsFailuresAndBackOff](ADR-ResultsFailuresAndBackOff.md) | The exit code decides the outcome and `mutants.out` the contents; a baseline failure is retried once, tool failures back off and the third publishes. |
| [ADR-InjectedTime](ADR-InjectedTime.md) | Every clock read and wait goes through `Clock` and `Pause`, deadlines inside the `CommandRunner`, so the loop is tested without sleeping. |
| [ADR-PlainBinaryPackaging](ADR-PlainBinaryPackaging.md) | A plain binary pointed at a roster, not a cargo subcommand. |
| [ADR-DeployKeyAndRuleset](ADR-DeployKeyAndRuleset.md) | A write deploy key and a ruleset blocking every branch but `mutants`, per repository. |
| [ADR-NoDependencies](ADR-NoDependencies.md) | The core depends on the standard library alone, because it runs beside every repository's key. |

## Template

```markdown
# ADR-<Name>

## Status
Accepted / Superseded by <ADR>.

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.
`N/A` if purely structural.

## Related
Links to other ADRs and architecture docs.
```

Fields that do not apply are marked `N/A` rather than padded. Each ADR is a snapshot of the decision as it stands
today, not a changelog — state the current shape as fact, don't narrate what an earlier version of this document used
to say.
