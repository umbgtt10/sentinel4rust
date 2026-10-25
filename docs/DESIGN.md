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
| 8. Ideas for later | [`ROADMAP.md`](ROADMAP.md), under consideration |
| 9. Implementation notes | the ADRs above, where each note became the decision: [`ADRs/ADR-NoDependencies.md`](ADRs/ADR-NoDependencies.md), keys and counts from the lists, publishing by plumbing, bare clones, cleanup on start, the baseline retry in the sentinel |

## Where it was wrong

The history copy is kept as it was written. Four of its statements did not hold for the code, and the documents
above say what does:

- **2.4** says publishing uses a second worktree and `git push origin HEAD:refs/heads/mutants`. The code writes the
  commit with plumbing alone -- `hash-object`, a temporary index, `write-tree`, `commit-tree` -- and pushes
  `<commit>:refs/heads/mutants`, with no worktree on `mutants` (`ADRs/ADR-ExactCommitOnlyMutantsPushed.md`).
- **2.6** says the exit codes were "checked against 27.1.0 in S5". The exit-code table matches cargo-mutants 27.1.0's
  `exit_code.rs`; `outcomes.json` was never checked, and is not read: keys and counts come from the `.txt` lists.
  Exits 2, 3 and 4 now run against the real 27.1.0 in stage 2 (`ADRs/ADR-ResultsFailuresAndBackOff.md`).
- **6** says the parser is tested "against `mutants.out` fixtures captured from real runs". When it was written, the
  tests wrote synthetic lists through a fake runner; lists captured from real runs of a failing baseline and of a
  looping mutant are now in `core/tests/fixtures/mutants_out/` (`ARCHITECTURE.md`, testing).
- **9** says stage 2 "has not been run yet". It ran and passed on the development laptop on 2026-10-24, against
  cargo-mutants 27.1.0, and runs in every stage 2.
