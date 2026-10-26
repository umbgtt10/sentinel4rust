# ADR — Crate filters, and accepted survivors, live in each repository

## Status

Accepted.

## Context

Not every crate in a workspace should be mutated, not every test should run, and some survivors are equivalent
mutants nobody will kill. Somebody has to say which.

## Decision

The repository says it, in `.cargo/mutants.toml` (`exclude_globs`, `exclude_re`, `test_workspace`,
`timeout_multiplier`) and with `#[mutants::skip]`; the roster says nothing about it, and a repository without
`.cargo/mutants.toml` is skipped.

## Forcing constraints / Evidence

Filters in the repository are reviewed like code and give the same result for a hand run of `cargo mutants`. A
repository that never said what to exclude would otherwise have everything mutated, including what was meant to be
left out.

## Rejected alternatives

- **Filters in the roster.** Unreviewed by the repository's owners, and a hand run would disagree with the published
  lists.
- **Filters on the `mutants` branch.** Mixes configuration into results.

## Consequences

Onboarding a repository needs a commit on its `main`. Before running, after the fetch, a visit is skipped with a
reason and publishes nothing when `main` has no root `Cargo.toml` (`not-a-crate`) or no `.cargo/mutants.toml`
(`no-filters`); a branch missing on the remote is `no-branch`; a manifest at `head` with a `path` dependency outside
the repository, which no checkout holds, is `path-outside-repo`, naming the manifest.

## Enforcement

`CrateReadiness::check` (`cli/src/system/git/crate_readiness.rs`) and its tests; the real-git test
`sync_without_cargo_toml_or_mutants_config_is_not_ready`; the sentinel test that not-ready repositories are skipped
without running.

## Related

- `ADR-ResultsFailuresAndBackOff.md`.
