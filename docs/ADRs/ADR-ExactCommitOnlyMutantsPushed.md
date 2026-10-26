# ADR — Runs are against an exact commit; only `mutants` is pushed

## Status

Accepted.

## Context

The tool holds a key that can write to every onboarded repository, and runs each repository's code. Whatever it
tests must be exactly the commit it names, and whatever it writes must be exactly one branch.

## Decision

Each run checks `head` out as a detached worktree nobody works in, and publishing writes the commit with git plumbing
alone and pushes `<commit>:refs/heads/mutants` and nothing else.

## Forcing constraints / Evidence

A detached worktree of a bare clone (`<work_dir>/runs/<name>`) is the commit, not someone's working copy.
cargo-mutants' output goes to `<work_dir>/out/<name>`, outside it, and its copies of the tree to `TMPDIR`,
`<work_dir>/tmp/<name>`. Publishing checks nothing out: each file is written
under `<work_dir>/stage/<name>` and stored with `hash-object`, added to a temporary index with `update-index`, and
`write-tree` and `commit-tree` (parent: the fetched `mutants`, if any; author and committer `sentinel4rust`) make the
commit. With no worktree on `mutants`, there is no branch checked out that a push could be made from by mistake.

## Rejected alternatives

- **A second worktree on `mutants` and `git push origin HEAD:refs/heads/mutants`.** What the design first said: it
  needs a checkout of `mutants` kept in step, and pushes whatever `HEAD` happens to be.
- **Testing in the clone's own working tree.** A bare clone has none, and a shared one carries state between runs.

## Consequences

The checkout, the output, the temporary files and the staging area are removed after every run, whatever the
outcome; on start, `runs/`, `stage/` and `tmp/` are emptied and `git worktree prune` runs in every clone. A push rejected because `mutants` moved is
logged and backed off, and the next visit reruns. Rulesets make "only `mutants`" hold even against a bug
(`ADR-DeployKeyAndRuleset.md`).

## Enforcement

`GitCli` (`cli/src/system/git/git_cli.rs`) names the branch in one private constant and pushes in one place;
`DetachedWorktree` and `PlumbingCommit` are tested against a scripted runner, and the real-git tests assert `main` is
byte-identical after every publish, worktrees are gone after a run, and a push after `mutants` moved is rejected.

## Related

- `ADR-NotAGate.md`, `ADR-OrphanMutantsBranch.md`, `ADR-DeployKeyAndRuleset.md`.
