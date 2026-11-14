# Architecture Decision Records

Each file records one decision that shaped this codebase, with the forces that made it and the alternatives it
displaced. They are not tiered: `sentinel4rust` is one service with a small enough decision surface that a flat list is
sufficient. Most began as sections of the original design, kept unchanged in `docs/history/[SUPERSEDED] DESIGN.md`.

## Index

| ADR | Decision |
|---|---|
| [ADR-NotAGate](ADR-NotAGate.md) | It answers which changes the tests would not notice after the fact, for every commit `main` reaches; nothing waits for it and it never touches `main`. |
| [ADR-StateLivesOnMutants](ADR-StateLivesOnMutants.md) | The record in the repository's folder of the results repository is the only state: current when it names `head` and the roster's `config_version`. |
| [ADR-SharedResultsRepository](ADR-SharedResultsRepository.md) | The watched repositories are only read, over anonymous https; results go to one results repository, one folder each, with the sentinel's only key. |
| [ADR-GithubAppDiscovery](ADR-GithubAppDiscovery.md) | A read-only GitHub App finds the repositories with a `.cargo/mutants.toml` and reads them with a token the sentinel mints; no onboarding per repository. |
| [ADR-OrphanMutantsBranch](ADR-OrphanMutantsBranch.md) | *Superseded by ADR-SharedResultsRepository.* Results lived on an orphan `mutants` branch in each repository. |
| [ADR-PositionFreeMutantKeys](ADR-PositionFreeMutantKeys.md) | A mutant is keyed by cargo-mutants' line without `:line:col`, with ordinals for duplicates; keys and counts come from the `.txt` lists. |
| [ADR-ExactCommitOnlyMutantsPushed](ADR-ExactCommitOnlyMutantsPushed.md) | Runs use a detached worktree at `head`; publishing uses plumbing alone and pushes the results repository's `main` and nothing else. |
| [ADR-FiltersLiveInEachRepository](ADR-FiltersLiveInEachRepository.md) | What is mutated and which survivors are accepted is said in the repository's `.cargo/mutants.toml`; without one it is skipped. |
| [ADR-PollingPerRepository](ADR-PollingPerRepository.md) | The pause is a polling interval per repository; the longest-waiting due repository goes first; one runs at a time. |
| [ADR-ResultsFailuresAndBackOff](ADR-ResultsFailuresAndBackOff.md) | The exit code decides the outcome and `mutants.out` the contents; a baseline failure is retried once, tool failures back off and the third publishes. |
| [ADR-InjectedTime](ADR-InjectedTime.md) | Every clock read and wait goes through `Clock` and `Pause`, deadlines inside the `CommandRunner`, so the loop is tested without sleeping. |
| [ADR-PlainBinaryPackaging](ADR-PlainBinaryPackaging.md) | A plain binary pointed at a roster, not a cargo subcommand. |
| [ADR-DeployKeyAndRuleset](ADR-DeployKeyAndRuleset.md) | *Superseded by ADR-SharedResultsRepository.* A write deploy key and rulesets per repository; the rulesets did not hold the key. |
| [ADR-NoDependencies](ADR-NoDependencies.md) | The service depends on nothing outside the repository but `serde_json` in `cli`, because it runs beside the keys. |
| [ADR-TechnologyFreeCore](ADR-TechnologyFreeCore.md) | The logic is a `no_std` core with no dependency and no technology; the binary and every implementation are `cli`, and cargo-mutants mutates the core alone. |

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
