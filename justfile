# Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the MIT License
# SPDX-License-Identifier: MIT

# Applies to both stages: `just export` sets it in the process just spawns
# recipe shells from, so every cargo command below sees it.
export RUSTFLAGS := "-D warnings"

# CI sets fmt to fail on drift instead of silently rewriting files a human
# isn't there to review; a local run still auto-fixes for convenience.
fmt_mode := if env('CI', '') != '' { '--check' } else { '' }

default:
    @just --list

# Every commit message gains core's grip and braintax, scored as staged, by the
# commit-msg hook in .githooks/. Once a clone.
hooks:
    git config core.hooksPath .githooks

# The lock file to the newest compatible version of every dependency, while
# the tools are in development. A cargo built-in, so stage 1 still needs none of
# the house tools; it does need the network. Commit what it moves on its own.
sync:
    cargo xtask sync

# Formatting, clippy and tests -- cargo built-ins only, so it works on a fresh checkout.
# --workspace on clippy/test so cli and xtask are covered too; `cargo fmt`
# formats every workspace member by default already.
stage1: sync
    cargo fmt {{fmt_mode}}
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# The gates beyond cargo's own, run by the xtask.
stage2:
    cargo xtask stage2

# What ripe4rust finds ripe to upgrade, the toolchain or a dependency: a plan, nothing applied.
ripe:
    cargo ripe4rust plan
