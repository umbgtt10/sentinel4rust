# ADR — The core has no dependencies

## Status

Accepted.

## Context

The service runs unattended beside every onboarded repository's deploy key. Every crate it depends on is code that
runs there too, and an update to any of them is a change nobody reviewed.

## Decision

The service depends on nothing outside this repository: `sentinel4rust-core` on no crate at all, `#![no_std]`, and the
`sentinel4rust` package on the core and the standard library alone; a dependency comes in only if it removes real
code. The development-only `xtask` may use the family's crates.

## Forcing constraints / Evidence

The tool needs little: process spawning, git and cargo-mutants through the command line, and a small TOML subset
(comments, strings, integers, booleans, `[[repo]]`) for the roster and `summary.toml`, in `core/src/model/toml/`.
The build machine had no crates.io access when it was written, which forced the choice; the supply chain kept it.

## Rejected alternatives

- **`toml`, `serde`, `clap`, `tempfile`.** Each saves code and adds a supply chain to the machine holding the keys.
- **`libc` for killing process groups.** `kill -KILL -- -<pgid>` as an external program does the same with no
  `unsafe` and no dependency; the tool is Linux-only for it.

## Consequences

The TOML subset parses only what the tool writes and reads; a roster using anything else fails with the line it
failed on. `rust-version` is 1.89, for `File::try_lock`. `xtask` uses `serde`, `serde_json` and `syn`, and `tempfile`
for its tests, from `[workspace.dependencies]`.

## Enforcement

`core/Cargo.toml`'s `[dependencies]` is empty, and stage 2's technology-free gate fails on any crate its manifest
names, in any section; `cli/Cargo.toml` names the core alone; cargo-shear in stage 2 flags any dependency declared and
unused; CLAUDE.md states the rule.

## Related

- `ADR-PlainBinaryPackaging.md`, `ADR-TechnologyFreeCore.md`.
