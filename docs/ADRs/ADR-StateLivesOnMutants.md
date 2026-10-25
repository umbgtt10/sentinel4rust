# ADR — State lives on `mutants`

## Status

Accepted.

## Context

The loop needs one fact per repository: has `main` moved since the last published result? A local database would
answer it, and would also be a second copy of the truth that can disagree with the branch, be lost with the disk,
and need migrating.

## Decision

The record on the repository's `mutants` branch is the only state: a record is **current** when its `summary.toml`
names `head` and the roster's `config_version`; the machine keeps nothing else but the in-memory schedule.

## Forcing constraints / Evidence

Everything a rerun needs is already published, so a second store adds nothing it can know. Bumping `config_version`
(after upgrading cargo-mutants, say) reruns every repository once without touching any branch.

## Rejected alternatives

- **A local database or state file.** Diverges from the branch the moment someone deletes or rewrites `mutants`, and
  makes a machine rebuild a migration.
- **Comparing against the last run started.** A run that dies half way would then count as done.

## Consequences

A reboot loses only the schedule and back-off (`patrol/`), which costs at most an early visit. A run that dies half
way publishes nothing and is simply rerun. Deleting `mutants` on the remote drops the record: the next visit reruns.
Upgrading cargo-mutants without bumping `config_version` leaves old records current (`docs/OPEN_POINTS.md`, point
13). A `summary.toml` that does not parse reads as no record (point 4).

The fetch uses explicit refspecs for the configured branch and, when `git ls-remote` lists it, `mutants`; clones are
full bare clones at `<work_dir>/repos/<name>.git`.

## Enforcement

`RecordCurrency::is_current` (`core/src/model/branch/record_currency.rs`) and its test; the sentinel tests for an
unchanged `main` and a `config_version` bump; `GitCli::sync` reading the record through `RecordReader`, tested on real
git, including a deleted `mutants` dropping the record.

## Related

- `ADR-OrphanMutantsBranch.md` — what the record is.
- `ADR-NotAGate.md`.
