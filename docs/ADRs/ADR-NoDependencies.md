# ADR — The core has no dependencies

## Status

Accepted.

## Context

The service runs unattended beside the deploy key of the results repository. Every crate it depends on is code that
runs there too, and an update to any of them is a change nobody reviewed.

## Decision

The service depends on nothing outside this repository: `sentinel4rust-core` on no crate at all, `#![no_std]`, and the
`sentinel4rust` package on the core, the standard library and `serde_json` alone; a dependency comes in only if it
removes real code. The development-only `xtask` may use the family's crates.

## Forcing constraints / Evidence

The tool needs little: process spawning, git and cargo-mutants through the command line, and a small TOML subset
(comments, strings, integers, booleans, `[[repo]]`) for the roster and `summary.toml`, in `core/src/model/toml/`.
The build machine had no crates.io access when it was written, which forced the choice; the supply chain kept it.

`serde_json` is the one exception, since 2026-11-14 (`ADR-GithubAppDiscovery`): the GitHub App's API answers in
JSON -- the installation token and its expiry, the pages of repositories -- and a JSON reader is real code the crate
removes. It was already in the workspace's lock file, for `xtask`, so the machine's supply chain gains no new crate.
RS256 for the App's JWT did not justify a crate: `openssl` signs it as an external program, as git and curl are.

## Rejected alternatives

- **`toml`, `serde`, `clap`, `tempfile`.** Each saves code and adds a supply chain to the machine holding the keys.
- **`libc` for killing process groups.** `kill -KILL -- -<pgid>` as an external program does the same with no
  `unsafe` and no dependency; the tool is Linux-only for it.

## Consequences

The TOML subset parses only what the tool writes and reads; a roster using anything else fails with the line it
failed on. `rust-version` is 1.91, for `str::ceil_char_boundary`, where it was 1.89, for `File::try_lock`. `xtask`
uses `serde`, `serde_json` and `syn`, and `tempfile` for its tests, from `[workspace.dependencies]`.

## Enforcement

`core/Cargo.toml`'s `[dependencies]` is empty, and stage 2's technology-free gate fails on any crate its manifest
names, in any section; `cli/Cargo.toml` names the core and `serde_json` alone; cargo-shear in stage 2 flags any dependency declared and
unused; cargo-deny checks what the lock file holds; CLAUDE.md states the rule.

## Related

- `ADR-PlainBinaryPackaging.md`, `ADR-TechnologyFreeCore.md`.
