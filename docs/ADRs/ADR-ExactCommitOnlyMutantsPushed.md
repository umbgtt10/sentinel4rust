# ADR — Runs are against an exact commit; only the results repository's `main` is pushed

## Status

Accepted.

## Context

The tool holds a key that can write the results repository, and runs each watched repository's code. Whatever it
tests must be exactly the commit it names, and whatever it writes must be exactly one branch of one repository.

## Decision

Each run checks `head` out as a detached worktree nobody works in, and publishing writes the commit with git plumbing
alone and pushes `<commit>:refs/heads/main` to the results repository and nothing else; nothing is ever pushed to a
watched repository.

## Forcing constraints / Evidence

A detached worktree of a bare clone (`<work_dir>/runs/<name>`) is the commit, not someone's working copy.
cargo-mutants' output goes to `<work_dir>/out/<name>`, outside it, and its copies of the tree to `TMPDIR`,
`<work_dir>/tmp/<name>`. Publishing checks nothing out: in the results repository's bare clone,
`<work_dir>/results.git`, the fetched tip's tree is read into a temporary index under `<work_dir>/stage/<name>`,
the repository's folder is removed from it with `update-index --force-remove` (the staging area standing in for the
work tree a bare clone lacks), each file is stored with `hash-object` and added with `update-index`, and
`write-tree` and `commit-tree` (parent: the fetched tip, if any; author and committer `sentinel4rust`) make the
commit. With no worktree on the results branch, there is no branch checked out that a push could be made from by
mistake.

## Rejected alternatives

- **A worktree on the results branch and `git push origin HEAD:refs/heads/main`.** What the design first said, for
  `mutants`: it needs a checkout kept in step, and pushes whatever `HEAD` happens to be.
- **Testing in the clone's own working tree.** A bare clone has none, and a shared one carries state between runs.

## Consequences

The checkout, the output, the temporary files and the staging area are removed after every run, whatever the
outcome; on start, `runs/`, `stage/` and `tmp/` are emptied and `git worktree prune` runs in every clone. A push rejected because the results branch moved
is logged and backed off, and the next visit reruns. Only the results repository holds the key, so even a bug can
write nothing else (`ADR-SharedResultsRepository.md`).

## Enforcement

`ResultsClone` (`cli/src/system/git/results_clone.rs`) pushes in one place, the branch named once in
`git_constants.rs`; `GitCli` never pushes. `DetachedWorktree` and `PlumbingCommit` are tested against a scripted
runner, and the real-git tests assert the watched repository's refs are identical after every publish, worktrees are
gone after a run, and a push after the results branch moved is rejected.

## Related

- `ADR-NotAGate.md`, `ADR-SharedResultsRepository.md`.
