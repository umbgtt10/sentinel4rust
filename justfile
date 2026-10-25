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

# Formatting, clippy and tests -- cargo built-ins only, so it works on a fresh checkout.
# --workspace on clippy/test so the xtask member is covered too; `cargo fmt`
# formats every workspace member by default already.
stage1:
    cargo fmt {{fmt_mode}}
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# The gates beyond cargo's own, run by the xtask.
stage2:
    cargo xtask stage2
