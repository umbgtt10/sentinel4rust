# ADR — State lives on `mutants`

## Status

Accepted.

## Context

The loop needs one fact per repository: has `main` moved since the last published result? A local database would
answer it, and would also be a second copy of the truth that can disagree with the published record, be lost with
the disk, and need migrating.

## Decision

The record in the repository's folder on the results repository's `main` (`ADR-SharedResultsRepository.md`) is the
only state: a record is **current** when its `summary.toml`
names `head` and the roster's `config_version` and, when the installed version is known, was made by the installed
cargo-mutants; the machine keeps nothing else but the in-memory schedule.

## Forcing constraints / Evidence

Everything a rerun needs is already published, so a second store adds nothing it can know. Bumping `config_version`
reruns every repository once without touching any record; so does upgrading cargo-mutants, whose version
`summary.toml` records and `EntryPoint` reads once on start.

## Rejected alternatives

- **A local database or state file.** Diverges from the record the moment someone deletes or rewrites a folder of
  the results repository, and makes a machine rebuild a migration.
- **Comparing against the last run started.** A run that dies half way would then count as done.

## Consequences

A reboot loses only the schedule and back-off (`patrol/`), which costs at most an early visit. A run that dies half
way publishes nothing and is simply rerun. Deleting a repository's folder, or the results repository's `main`, drops its record: the next visit
reruns.
An installed version that cannot be read decides nothing, so a broken install does not rerun every repository. A
`summary.toml` that does not parse is its own state, `unreadable`: the record may be one a hand edit or a newer
tool broke, so it is never run over; the repository backs off to the cap, is logged once, and recovers when the record
reads again. One in a schema this version does not know reads as no record, and the commit is run again.

The fetch uses an explicit refspec for the configured branch; clones are full bare clones at
`<work_dir>/repos/<name>.git`. The results repository is one more, at `<work_dir>/results.git`, whose `main` is
fetched on every visit when `git ls-remote` lists it.

## Enforcement

`RecordCurrency::is_current` (`core/src/model/branch/record_currency.rs`) and its test; the sentinel tests for an
unchanged `main`, a `config_version` bump and an unreadable record; `GitCli::sync` reading the record through
`ResultsClone` and `RecordReader` into a `RecordState`, tested on real git, including a deleted results `main`
dropping the record and a garbage `summary.toml` read as unreadable.

## Related

- `ADR-SharedResultsRepository.md` — what the record is and where it lives.
- `ADR-NotAGate.md`.
