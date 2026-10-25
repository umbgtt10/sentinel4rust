# sentinel4rust -- design (moved)

The design the implementation was built from, revision 3.1, is now
[`docs/history/[SUPERSEDED] DESIGN.md`](history/%5BSUPERSEDED%5D%20DESIGN.md), unchanged. It is not kept up to
date. Where each of its sections lives now:

| section | now |
|---|---|
| intro, "not a gate" | [`ADRs/ADR-NotAGate.md`](ADRs/ADR-NotAGate.md) |
| 1. The loop | [`ARCHITECTURE.md`](ARCHITECTURE.md), the pipeline; [`ADRs/ADR-PollingPerRepository.md`](ADRs/ADR-PollingPerRepository.md) |
| 2.1 State lives on `mutants` | [`ADRs/ADR-StateLivesOnMutants.md`](ADRs/ADR-StateLivesOnMutants.md) |
| 2.2 `mutants` is an orphan branch | [`ADRs/ADR-OrphanMutantsBranch.md`](ADRs/ADR-OrphanMutantsBranch.md) |
| 2.3 Mutants are identified without line numbers | [`ADRs/ADR-PositionFreeMutantKeys.md`](ADRs/ADR-PositionFreeMutantKeys.md) |
| 2.4 Runs are against an exact commit; only `mutants` is pushed | [`ADRs/ADR-ExactCommitOnlyMutantsPushed.md`](ADRs/ADR-ExactCommitOnlyMutantsPushed.md) |
| 2.5 Crate filters, and accepted survivors, live in each repository | [`ADRs/ADR-FiltersLiveInEachRepository.md`](ADRs/ADR-FiltersLiveInEachRepository.md) |
| 2.6 Reading results, failures and timeouts | [`ADRs/ADR-ResultsFailuresAndBackOff.md`](ADRs/ADR-ResultsFailuresAndBackOff.md) |
| 2.7 Time is injected | [`ADRs/ADR-InjectedTime.md`](ADRs/ADR-InjectedTime.md) |
| 2.8 Packaging | [`ADRs/ADR-PlainBinaryPackaging.md`](ADRs/ADR-PlainBinaryPackaging.md) |
| 3. Configuration | [`ARCHITECTURE.md`](ARCHITECTURE.md), configuration; [`../deploy/roster.example.toml`](../deploy/roster.example.toml) |
| 4. Architecture | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| 5. The machine, 5.2 Setting it up | [`../deploy/SETUP.md`](../deploy/SETUP.md) |
| 5.1 Security model | [`ADRs/ADR-DeployKeyAndRuleset.md`](ADRs/ADR-DeployKeyAndRuleset.md); [`OPEN_POINTS.md`](OPEN_POINTS.md), points 5 and 15 |
| 6. Testing | [`ARCHITECTURE.md`](ARCHITECTURE.md), testing |
| 7. Implementation slices | done; the restructure that followed is in [`ROADMAP.md`](ROADMAP.md) |
| 8. Ideas for later | [`ROADMAP.md`](ROADMAP.md), ideas for later |
| 9. Implementation notes | the ADRs above, where each note became the decision: [`ADRs/ADR-NoDependencies.md`](ADRs/ADR-NoDependencies.md), keys and counts from the lists, publishing by plumbing, bare clones, cleanup on start, the baseline retry in the sentinel |
