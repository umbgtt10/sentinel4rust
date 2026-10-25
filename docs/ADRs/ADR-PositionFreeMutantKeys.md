# ADR — Mutants are identified without line numbers

## Status

Accepted.

## Context

cargo-mutants names a mutant `file:line:col: description`. One line added at the top of a file shifts every position
below it, so a list keyed by position would show every survivor in the file as removed and re-added, and `new`/`gone`
would mean nothing.

## Decision

A mutant's **key** is the line cargo-mutants wrote with `:line:col` removed (`src/lib.rs: replace == with != in
Parser::next`), with an ordinal (`#2`) in source order when the same key occurs more than once in a file; keys and
counts come from `mutants.out`'s `.txt` lists.

## Forcing constraints / Evidence

The description already names the function, so position adds nothing a reader needs. The `.txt` lists are the
format cargo-mutants 27.1.0 writes one mutant per line; `outcomes.json`'s field names could not be checked against
that version when the code was written, and the lists made it unnecessary. `new` and `gone` are the set differences
with the previous record's `missed.txt`; timeouts never count as new.

## Rejected alternatives

- **Keys with positions.** Every edit above a survivor reports it as gone and new.
- **Keys from `outcomes.json`.** A second format to track across cargo-mutants versions for no extra information.
- **Hashing the mutated source.** Stable under moves, but unreadable in a diff.

## Consequences

The published lists diff cleanly as code moves. Ordinals can shift: removing the first of two identical keys in a
file renumbers the second, which then reads as one gone and one new (`docs/OPEN_POINTS.md`, point 16). The positions
are kept, as written, under `located/`.

## Enforcement

`Located` and `MutantKey` (`core/src/model/`) with `core/tests/model/mutant_key_tests.rs` (keys survive lines inserted
above, duplicates get ordinals, colons in paths are kept) and `MutantKey::changes` for the set differences;
`MutantsOutLists` reads the lists, and a completed exit without `caught.txt` and `missed.txt` is a tool failure.

## Related

- `ADR-OrphanMutantsBranch.md`, `ADR-ResultsFailuresAndBackOff.md`.
