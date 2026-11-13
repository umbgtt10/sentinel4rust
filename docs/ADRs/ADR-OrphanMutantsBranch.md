# ADR — `mutants` is an orphan branch

## Status

Accepted.

## Context

The results need a home that travels with the repository, is visible on the host, keeps its own history and cannot
be confused with the code.

## Decision

The results live on an orphan branch, `mutants`, that shares no history with `main`; each publication is one commit
that overwrites the branch's files.

```
mutants (orphan)
  README.md       what this branch is; generated; "do not merge"
  summary.toml    schema = 1, commit, config_version, kind = "full", started/finished UTC, duration_seconds,
                  outcome, counts (caught, missed, timeout, unviable), new/gone, cargo-mutants and rustc
                  versions, arguments; reason and log tail on failures
  missed.txt      one surviving mutant per line, by key, sorted
  timeout.txt     one timed-out mutant per line, by key, sorted (absent when none)
  located/        the same lists as cargo-mutants wrote them, with file:line:col
```

Commit messages: `Mutants of <short hash>: <missed> missed (+<new> -<gone>), <caught> caught`, then
`, <timeout> timed out` when any did, or `Mutants of <short hash>: baseline failed` / `tool failed`. A run's
timeouts are in its message so that one whose mutants mostly timed out does not read as a clean run. Failures write `README.md` and `summary.toml` only.

## Forcing constraints / Evidence

Sharing no history, it cannot be merged by accident, and `git log -p mutants` is the history of the survivors with
nothing else in it.

## Rejected alternatives

- **Files on `main`.** Every run would be a commit on the branch under test, moving `main` and triggering itself.
- **A branch forked from `main`.** Mergeable by accident, and its log is `main`'s.
- **Releases, issues or a web page.** Outside git: no diff of the survivors between two commits.

## Consequences

Each run overwrites everything; git keeps the past. On a public repository the branch is public
(`ADR-DeployKeyAndRuleset.md`). The schema is versioned: a `summary.toml` with an unknown `schema` reads as no
record and the commit is run again.

## Enforcement

`BranchLayout` (`core/src/model/branch/branch_layout.rs`) owns the file list and the messages, `SummaryToml` the
schema; both are tested in `core/tests/model/branch/`. `publish_first_creates_an_orphan_branch_and_leaves_main_alone`
checks on real git that the first publication makes `mutants` a single commit and leaves `main` as it was.

## Related

- `ADR-StateLivesOnMutants.md`, `ADR-PositionFreeMutantKeys.md`, `ADR-ExactCommitOnlyMutantsPushed.md`.
