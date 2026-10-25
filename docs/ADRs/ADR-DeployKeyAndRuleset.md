# ADR — A deploy key and a ruleset per repository

## Status

Accepted.

## Context

The tool reads every onboarded repository and must write exactly one branch in each, `mutants`. GitHub cannot scope a
credential to a branch.

## Decision

Two layers per repository: a **deploy key with write access**, reached through a per-repository SSH host alias, and
a **ruleset** that blocks creating, updating, deleting and force-pushing every branch except `mutants`, with no
bypass for deploy keys.

## Forcing constraints / Evidence

GitHub will not accept one deploy key on two repositories, hence one key and one host alias (`github-<repo>`) each.
The ruleset makes "only `mutants`" hold even against a bug in the tool, and `deploy/verify-access.sh` checks that it
does before a repository is relied on.

## Rejected alternatives

- **A personal access token or the owner's key.** Writes everything everywhere.
- **The tool's own discipline alone.** Holds only as long as the code has no bug.

## Consequences

Accepted, and recorded in `docs/OPEN_POINTS.md`: a leaked key reads its repository; survivors and log tails are
published on public repositories too (point 15). Every repository's code runs where cargo-mutants runs; with
`run_unit` set, that is a unit of its own with `~/.ssh` inaccessible, started and stopped through a polkit rule that
allows the `sentinel` user to start and stop that unit and nothing else, while git and the keys stay in the service
(point 5; to be proven on the machine with `deploy/verify-isolation.sh`).
Rulesets on private repositories need a paid GitHub plan; without one, those rely on the tool pushing only `mutants`
(`ADR-ExactCommitOnlyMutantsPushed.md`). Commits on `mutants` are unsigned, so signed commits may be required on
`main` only. Revoking is deleting the key on GitHub and the key file.

## Enforcement

`deploy/verify-access.sh` (reading works; pushing anything but `mutants` is rejected); `deploy/SETUP.md` for the steps;
`GitCli`'s single push refspec.

## Related

- `ADR-ExactCommitOnlyMutantsPushed.md`, `ADR-NotAGate.md`.
