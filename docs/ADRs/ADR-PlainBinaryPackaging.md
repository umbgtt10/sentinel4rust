# ADR — A plain binary

## Status

Accepted.

## Context

The sibling `*4rust` tools are cargo subcommands: run inside a project, about that project. This one is run by a
service manager, pointed at a list of repositories it clones itself.

## Decision

The package builds a plain binary, `sentinel4rust`, taking `--roster <file>` and optionally `--once`.

## Forcing constraints / Evidence

Nothing about a run depends on the directory it is started in; a `cargo sentinel4rust` would suggest otherwise.

## Rejected alternatives

- **A cargo subcommand.** Implies a project to run in, and puts the subcommand name in `argv[1]` for nothing.

## Consequences

It is installed from the repository with `cargo install --path cli --locked` and run by the systemd unit in
`deploy/`. It takes a lock on `<work_dir>/sentinel4rust.lock`, so a manual `--once` beside the service exits at once
with a message. Exit codes: 0 on success, 1 when it cannot start or a `--once` visit backed off or found a repository
unreachable, 2 for a bad command line.

## Enforcement

`cli/src/main.rs` is the binary, one line handing the arguments to `EntryPoint`; `cli/tests/main_tests.rs` drives it end to end, and
`cli/tests/invocation/entry_point_tests.rs` pins each exit code.

## Related

- `ADR-NoDependencies.md`, `deploy/SETUP.md`.
