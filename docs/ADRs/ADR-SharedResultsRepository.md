# ADR — One shared results repository; the watched repositories are only read

## Status

Accepted. Supersedes `ADR-DeployKeyAndRuleset.md` and `ADR-OrphanMutantsBranch.md`.

## Context

The tool reads every watched repository and must publish its results somewhere. GitHub cannot scope a credential
to a branch, so any key that writes a watched repository can write all of it: `main`, every other branch, every tag.
The earlier design confined such a key with rulesets; on the first real onboarding the rulesets did not confine it.

## Decision

The watched repositories are read over anonymous https and never written; every result is published to one results
repository, `umbgtt10/mutants`, on its `main`, one folder per watched repository, with the sentinel's only key, a
write deploy key on that repository and nothing else.

```
umbgtt10/mutants, main
  README.md            what this repository is; generated; "do not edit by hand"
  <repo>/summary.toml  schema = 1, commit, config_version, kind = "full", started/finished UTC, duration_seconds,
                       outcome, counts (caught, missed, timeout, unviable), new/gone, cargo-mutants and rustc
                       versions, arguments, test_timeout_seconds when cargo-mutants' debug log states it; reason and
                       log tail on failures
  <repo>/missed.txt    one surviving mutant per line, by key, sorted
  <repo>/timeout.txt   one timed-out mutant per line, by key, sorted (absent when none)
  <repo>/located/      the same lists as cargo-mutants wrote them, with file:line:col
```

Each publication is one commit that replaces exactly its repository's folder and rewrites the root `README.md`,
leaving every other folder and file as it was: the fetched tip's tree is read into a temporary index, the folder's
paths are removed from it, the new files are added, and `commit-tree` makes the commit with the tip as its parent.
Commit messages: `<repo>: Mutants of <short hash>: <missed> missed (+<new> -<gone>), <caught> caught`, then
`, <timeout> timed out` when any did, or `<repo>: Mutants of <short hash>: baseline failed` / `tool failed`.
Failures write `README.md` and `<repo>/summary.toml` only. `git log -p -- <repo>/` is one repository's history of
survivors.

The roster names the results repository in a required root key, `results_url` (for example
`git@github-mutants:umbgtt10/mutants.git`, through the host alias `github-mutants` and the key
`~sentinel/.ssh/sentinel_mutants`); each `[[repo]]` url is `https://github.com/<owner>/<repo>.git`.

## Forcing constraints / Evidence

On 2026-11-14 the first real onboarding, of braintax4rust, created both rulesets of the earlier design --
`sentinel: only mutants` on every branch but `mutants`, `sentinel: no tags` on every tag, creation, update, deletion
and force-push restricted, the Repository admin role the only bypass, no deploy keys -- and then a write deploy key.
GitHub listed both rulesets as active and as applying to the probe branch, yet `verify-access.sh` pushed the probe
branch **and** the probe tag with the deploy key, and both were accepted. GitHub's documentation says a deploy key
with write access can do what an admin of an organisation's repository, or a collaborator on a personal one, can:
the admin-role bypass most likely let the key through. A ruleset without that bypass would refuse the owner as well,
on every push and every release tag. The key, both rulesets and the probes were deleted the same day; no other
repository had been onboarded.

A key cannot be kept off a repository's refs by rules on that repository, then; it can be kept off by never being
put there. A repository that holds nothing but results is the one place a write key can do no harm worth guarding.
GitHub will not accept one deploy key on two repositories, so one results repository means one key.

## Rejected alternatives

- **Rulesets with the admin role as the only bypass.** The evidence above: the write deploy key got through.
- **Rulesets with no bypass.** They would hold the key, and the owner too: every push to `main`, every branch and
  every release tag would need a ruleset switched off and on again.
- **A results repository per watched repository.** Eleven repositories and eleven keys to keep, for no gain in
  confinement: a leaked key writes results either way.
- **An orphan branch per watched repository in the shared repository.** Closest to the earlier code, but the owner
  chose folders on one branch: one place to browse, one ref to push.

## Consequences

A leaked key writes the results repository and nothing else; the watched repositories are reached with no
credential at all. A private repository is read with a read-only deploy key of its own, which cannot push at all, and
`deploy/onboard.sh` marks one `BLOCKED` while the results repository is public. A private git dependency is served to
the run unit from the sentinel's own clone (`deploy/SETUP.md` section 2), so the run unit still holds no key. Every watched repository's
survivors, and on failures a log tail with paths, are in one place, which the owner made private on 2026-11-14, the day
it was created: nothing in the design depends on its visibility (`deploy/SETUP.md` section 7). A private watched
repository's results can go to the same place without exposing anything. All
publications share one branch: a push rejected because `main` moved -- a hand edit, or a second sentinel -- is a
`PublishError`, logged and backed off, and the next visit fetches the new tip and publishes again. A hand-made file
outside the folders survives every publication. The schema is versioned: a `summary.toml` with an unknown `schema`
reads as no record and the commit is run again.

## Enforcement

`BranchLayout` (`core/src/model/branch/branch_layout.rs`) owns the file list under the folder and the messages,
tested in `core/tests/model/branch/`; `TomlRoster` requires `results_url`. `ResultsClone`
(`cli/src/system/git/results_clone.rs`) is the only part that pushes, and only `<commit>:refs/heads/main` of the
results repository; its real-git tests check a second repository keeps the first one's folder, a folder is replaced
whole, a hand-made file is kept and a push after the branch moved is rejected. `GitCli`'s tests assert the watched
repository's refs are identical after a publish, and the end-to-end tests that the watched repository never gains a
`mutants` branch. `deploy/verify-access.sh` proves, before a repository is relied on, that each watched repository
reads with no credential and that the key writes the results repository; `deploy/onboard.sh` refuses a watched
repository that still carries a deploy key titled `sentinel4rust`.

## Related

- `ADR-StateLivesOnMutants.md` -- what the record is for; `ADR-ExactCommitOnlyMutantsPushed.md` -- how it is written.
- `ADR-DeployKeyAndRuleset.md`, `ADR-OrphanMutantsBranch.md` -- the design this replaces.
