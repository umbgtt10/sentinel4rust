#!/bin/sh
# Verifies the security model for one repository (deploy/SETUP.md): reading works, and writing anything but
# `mutants` is rejected by the rulesets. Run as the `sentinel` user.
#
#   deploy/verify-access.sh [--include-main] git@github-<repo>:umbgtt10/<repo>.git
#
# By default it only probes by creating a branch and a tag, which is harmless: a missing ruleset lets them through,
# and the script deletes them again. The rulesets in deploy/SETUP.md target every branch except `mutants`, and every
# tag, so a rejected creation means the rule is in force. With --include-main it also tries to update and delete
# `main`; if the ruleset is missing those succeed, and the script pushes `main` back to where it was at once. Every
# write to `main` carries a lease on what the script last saw there, so a commit someone else pushed to `main` in
# between is never overwritten or deleted: when a lease refuses, the script says so and leaves `main` alone.

set -u
include_main=0
if [ "${1:-}" = "--include-main" ]; then include_main=1; shift; fi
url="${1:?usage: verify-access.sh [--include-main] <repo url>}"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
fail=0
g() { git -C "$scratch" "$@"; }

echo "== reading"
original="$(git ls-remote "$url" refs/heads/main 2>/dev/null | cut -f1)"
if [ -z "$original" ]; then
  echo "FAIL  cannot read main from $url"; exit 1
fi
echo "ok    main is at $original"

g init --quiet
g remote add origin "$url"
# One commit is enough to probe with. --include-main takes main's whole history: a remote refuses a shallow
# push, and putting main back after the probe deleted it is one.
depth="--depth 1"
if [ "$include_main" -eq 1 ]; then depth=""; fi
g fetch --quiet $depth origin main || { echo "FAIL  cannot fetch main"; exit 1; }
probe="$(g -c user.name=probe -c user.email=probe@localhost \
  commit-tree -p FETCH_HEAD -m 'sentinel4rust access probe -- must be rejected' 'FETCH_HEAD^{tree}')"

echo "== writing anything but mutants must be rejected"
if g push --quiet origin "$probe:refs/heads/sentinel4rust-probe" >/dev/null 2>&1; then
  echo "FAIL  creating a branch was ACCEPTED"; fail=1
  g push --quiet origin --delete sentinel4rust-probe >/dev/null 2>&1 \
    && echo "      (probe branch deleted again)"
else
  echo "ok    creating a branch rejected"
fi
if g push --quiet origin "$probe:refs/tags/sentinel4rust-probe" >/dev/null 2>&1; then
  echo "FAIL  creating a tag was ACCEPTED"; fail=1
  g push --quiet origin --delete refs/tags/sentinel4rust-probe >/dev/null 2>&1 \
    && echo "      (probe tag deleted again)"
else
  echo "ok    creating a tag rejected"
fi

if [ "$include_main" -eq 1 ]; then
  # Puts main back at $original, if main still holds what the probe left there ($1: the probe commit, or nothing
  # when the probe deleted main). Fails when it is not back.
  restore() {
    if g push --quiet --force-with-lease="refs/heads/main:$1" origin "$original:refs/heads/main" >/dev/null 2>&1
    then
      echo "      (main restored to $original)"
    elif [ "$(git ls-remote "$url" refs/heads/main 2>/dev/null | cut -f1)" != "$1" ]; then
      echo "      !! main moved again after the probe; the lease kept that push and main was NOT restored:"
      echo "         check main, and push $original to it by hand only if the probe is still on it"
      return 1
    else
      echo "      !! could not restore main: push $original to main by hand"
      return 1
    fi
  }
  at_original=1
  if g push --quiet origin "$probe:refs/heads/main" >/dev/null 2>&1; then
    echo "FAIL  updating main was ACCEPTED"; fail=1; restore "$probe" || at_original=0
  else
    echo "ok    updating main rejected"
  fi
  if [ "$at_original" -eq 0 ]; then
    echo "skip  deleting main: main is not at $original, and deleting it could discard someone's push"
  elif out="$(g push --force-with-lease="refs/heads/main:$original" origin --delete main 2>&1)"; then
    echo "FAIL  deleting main was ACCEPTED"; fail=1; restore "" || true
  elif printf '%s' "$out" | grep -q "stale info"; then
    echo "skip  deleting main: main moved after $original, and deleting it could discard someone's push"
  else
    echo "ok    deleting main rejected"
  fi
fi

echo "== writing mutants is checked by the first run: sentinel4rust --roster <file> --once"
if [ "$fail" -ne 0 ]; then
  echo "RESULT: FAIL -- fix the rulesets (deploy/SETUP.md) before relying on this repository"
  exit 1
fi
echo "RESULT: ok"
