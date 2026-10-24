#!/bin/sh
# Verifies the security model for one repository (design 5.2): reading works, and writing anything but
# `mutants` is rejected by the ruleset. Run as the `sentinel` user.
#
#   deploy/verify-access.sh [--include-main] git@github-<repo>:umbgtt10/<repo>.git
#
# By default it only probes by creating a branch, which is harmless: a missing ruleset lets it through, and
# the script deletes it again. The ruleset in design 5.2 targets every branch except `mutants`, so a rejected
# creation means the rule is in force. With --include-main it also tries to update and delete `main`; if the
# ruleset is missing those succeed, and the script immediately force-pushes `main` back to where it was.

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
g fetch --quiet --depth 1 origin main || { echo "FAIL  cannot fetch main"; exit 1; }
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

if [ "$include_main" -eq 1 ]; then
  restore() {
    g push --quiet --force origin "$original:refs/heads/main" >/dev/null 2>&1 \
      && echo "      (main restored to $original)" \
      || echo "      !! could not restore main: push $original to main by hand"
  }
  if g push --quiet origin "$probe:refs/heads/main" >/dev/null 2>&1; then
    echo "FAIL  updating main was ACCEPTED"; fail=1; restore
  else
    echo "ok    updating main rejected"
  fi
  if g push --quiet origin --delete main >/dev/null 2>&1; then
    echo "FAIL  deleting main was ACCEPTED"; fail=1; restore
  else
    echo "ok    deleting main rejected"
  fi
fi

echo "== writing mutants is checked by the first run: sentinel4rust --roster <file> --once"
if [ "$fail" -ne 0 ]; then
  echo "RESULT: FAIL -- fix the ruleset (design 5.2) before relying on this repository"
  exit 1
fi
echo "RESULT: ok"
