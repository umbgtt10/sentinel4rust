#!/bin/sh
# Verifies the access model (deploy/SETUP.md, docs/ADRs/ADR-SharedResultsRepository.md): every watched repository
# reads with no credential, and the sentinel's key writes the results repository. Run as the `sentinel` user.
#
#   deploy/verify-access.sh <results url> [<watched repository url>...]
#
# Writing is proven by creating a branch, sentinel4rust-probe, on the results repository and deleting it again; its
# `main` is never touched. Nothing is ever written to a watched repository: the sentinel holds no credential for one.

set -u
results="${1:?usage: verify-access.sh <results url> [<watched repository url>...]}"
shift
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
fail=0
g() { git -C "$scratch" "$@"; }
export GIT_TERMINAL_PROMPT=0

echo "== reading the watched repositories, with no credential"
for url in "$@"; do
  head="$(git ls-remote "$url" refs/heads/main 2>/dev/null | cut -f1)"
  if [ -n "$head" ]; then
    echo "ok    $url: main is at $head"
  else
    echo "FAIL  cannot read main from $url"; fail=1
  fi
done

echo "== writing the results repository, with the key"
g init --quiet
g remote add origin "$results"
if ! listing="$(git ls-remote "$results" refs/heads/main 2>&1)"; then
  echo "FAIL  cannot read $results: $listing"
  echo "RESULT: FAIL -- check the deploy key and the host alias (deploy/SETUP.md) before relying on it"
  exit 1
fi
if [ -n "$listing" ]; then
  g fetch --quiet --depth 1 origin main || { echo "FAIL  cannot fetch main from $results"; exit 1; }
  echo "ok    main is at $(g rev-parse FETCH_HEAD)"
  parent="-p FETCH_HEAD"
  tree="FETCH_HEAD^{tree}"
else
  echo "ok    no main yet: the first publication creates it"
  parent=""
  tree="$(g hash-object -t tree -w /dev/null)"
fi
# shellcheck disable=SC2086
probe="$(g -c user.name=probe -c user.email=probe@localhost \
  commit-tree $parent -m 'sentinel4rust access probe -- deleted at once' "$tree")"
if g push --quiet origin "$probe:refs/heads/sentinel4rust-probe" >/dev/null 2>&1; then
  echo "ok    creating a branch accepted"
  if g push --quiet origin --delete sentinel4rust-probe >/dev/null 2>&1; then
    echo "ok    the probe branch deleted again"
  else
    echo "FAIL  could not delete the probe branch sentinel4rust-probe: delete it by hand"; fail=1
  fi
else
  echo "FAIL  creating a branch was REJECTED: the key cannot write $results"; fail=1
fi

if [ "$fail" -ne 0 ]; then
  echo "RESULT: FAIL -- fix the access (deploy/SETUP.md) before relying on it"
  exit 1
fi
echo "RESULT: ok"
