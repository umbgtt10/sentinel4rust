#!/usr/bin/env bash
# Puts the sentinel live (deploy/SETUP.md section 3, deploy/PROVISION.md): with the service stopped, one
# `sentinel4rust --once` pass as sentinel, its journal lines shown; then, only if no visit failed,
# `systemctl enable --now sentinel4rust`, and a check that the service is active and stays so.
#
# The pass runs in a transient unit, sentinel4rust-first-pass, with the service's user, environment and limits, so it
# outlives this script. Interrupted -- by a timeout, a closed session -- run it again: it waits for the pass still
# running, then makes a second one, quick because every repository the first one published is current by then, and
# that one's result decides. Once the service is enabled and active, it only says so.
#
#   sudo deploy/go-live.sh [--dry-run]
#
#   --dry-run  print every action without doing any; needs no root

set -euo pipefail

readonly SENTINEL_HOME=/home/sentinel
readonly SERVICE=sentinel4rust
readonly PASS_UNIT=sentinel4rust-first-pass
readonly SETTLE_SECONDS=20
readonly cargo_bin="$SENTINEL_HOME/.cargo/bin"
readonly binary="$cargo_bin/sentinel4rust"
readonly roster="$SENTINEL_HOME/.config/sentinel4rust/roster.toml"

dry_run=0
blocked=0

say() { printf '%s\n' "$*"; }
ok() { say "ok    $*"; }
note() { say "note  $*"; }
failed=0
fail() {
  say "FAIL  $*" >&2
  failed=1
  exit 1
}
finish() {
  local status=$?
  if [ "$status" -ne 0 ] && [ "$failed" -eq 0 ]; then
    say "FAIL  stopped by the failing command above (exit $status): fix it and re-run" >&2
  fi
}
trap finish EXIT

usage() {
  awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "${BASH_SOURCE[0]}"
  exit 2
}

# require <message> <command...>: stops when the check fails; a dry run notes it and goes on.
require() {
  local message=$1
  shift
  if "$@" > /dev/null 2>&1; then
    return
  fi
  [ "$dry_run" -eq 1 ] || fail "$message"
  say "BLOCKED $message"
  blocked=1
}

# step <description> <command...>: does it, or only says it would.
step() {
  local what=$1
  shift
  if [ "$dry_run" -eq 1 ]; then
    say "would $what"
  else
    say "do    $what"
    "$@"
  fi
}

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) dry_run=1 ;;
    --help | -h) usage ;;
    *) say "unknown argument $1" >&2; usage ;;
  esac
  shift
done

if [ "$(id -u)" -ne 0 ]; then
  [ "$dry_run" -eq 1 ] || fail "run it with sudo: sudo $0"
  note "a dry run as $(id -un), not root: what the sentinel's home hides shows as blocked"
fi

# The pass's journal lines -- the start-up line and one per visit -- since an epoch second.
journal_since() {
  journalctl "_SYSTEMD_UNIT=$1.service" --since "@$2" -o cat --no-pager 2> /dev/null | sed 's/^/      /' || true
}

has_repo() {
  grep -Eq '^[[:space:]]*\[\[repo\]\]' "$roster"
}

work_dir() {
  [ -r "$roster" ] || return 0
  sed -n 's/^[[:space:]]*work_dir[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$roster" | sed -n 1p
}

preflight() {
  say "== preflight"
  require "there is no sentinel user: run deploy/provision.sh first" id sentinel
  require "$binary is missing: run deploy/provision.sh first" test -x "$binary"
  require "the service unit is not installed: run deploy/provision.sh first" \
    test -f "/etc/systemd/system/$SERVICE.service"
  require "$roster is missing: run deploy/provision.sh first" test -f "$roster"
  require "$roster has no [[repo]]: onboard one first, deploy/onboard.sh" has_repo
  [ "$blocked" -eq 1 ] || ok "sentinel, $binary, the unit and a roster with repositories"
}

# The service's user, environment and limits, as deploy/sentinel4rust.service sets them.
pass_command() {
  local dir
  dir="$(work_dir)"
  pass=(systemd-run "--unit=$PASS_UNIT" "--description=sentinel4rust: the first --once pass (deploy/go-live.sh)"
    --uid=sentinel --gid=sentinel "--working-directory=$SENTINEL_HOME"
    "--setenv=PATH=$cargo_bin:/usr/local/bin:/usr/bin:/bin"
    -p MemoryMax=80% -p TasksMax=8192 -p Nice=5
    -p NoNewPrivileges=yes -p PrivateTmp=yes -p ProtectSystem=strict "-p" "ReadWritePaths=$SENTINEL_HOME")
  if [ -n "$dir" ] && [[ "$dir" != "$SENTINEL_HOME"/* ]]; then
    pass+=(-p "ReadWritePaths=$dir")
  fi
  pass+=(--collect --wait --quiet "$binary" --roster "$roster" --once)
}

wait_for_running_pass() {
  local waited=0
  while systemctl is-active --quiet "$PASS_UNIT" 2> /dev/null; do
    if [ $((waited % 600)) -eq 0 ]; then
      say "wait  a pass is running ($((waited / 60)) min so far here); its last line:"
      journalctl "_SYSTEMD_UNIT=$PASS_UNIT.service" -n 1 -o cat --no-pager 2> /dev/null | sed 's/^/      /' || true
    fi
    sleep 30
    waited=$((waited + 30))
  done
  [ "$waited" -eq 0 ] || ok "the running pass ended; a second pass decides"
}

first_pass() {
  say "== the first pass, sentinel4rust --once, with the service stopped"
  local state
  state="$(systemctl is-active "$SERVICE" 2> /dev/null || true)"
  case "$state" in
    active | activating | reloading | deactivating | refreshing)
      step "stop $SERVICE ($state), which would take the work directory's lock" systemctl stop "$SERVICE"
      ;;
  esac
  pass_command
  if [ "$dry_run" -eq 1 ]; then
    say "would run, waiting for it: ${pass[*]}"
    return
  fi
  wait_for_running_pass
  local start status=0
  start="$(date +%s)"
  say "do    run, waiting for it: ${pass[*]}"
  say "      (it can take hours; follow it with journalctl -u $PASS_UNIT -f)"
  "${pass[@]}" || status=$?
  say "      its journal lines:"
  journal_since "$PASS_UNIT" "$start"
  [ "$status" -eq 0 ] || fail "the pass exited $status: a visit failed (the lines above say which) or it could not
      start. The service stays off; fix it, then run deploy/go-live.sh again"
  ok "no visit failed"
}

enable_service() {
  say "== the service"
  local start
  start="$(date +%s)"
  step "enable and start $SERVICE" systemctl enable --now "$SERVICE"
  if [ "$dry_run" -eq 1 ]; then
    say "would check, after ${SETTLE_SECONDS}s, that $SERVICE is active and has not restarted"
    return
  fi
  sleep "$SETTLE_SECONDS"
  journal_since "$SERVICE" "$start"
  systemctl is-active --quiet "$SERVICE" || fail "$SERVICE is not active: journalctl -u $SERVICE -n 50"
  [ "$(systemctl show -p NRestarts --value "$SERVICE")" = "0" ] \
    || fail "$SERVICE restarted after it started: journalctl -u $SERVICE -n 50"
  ok "$SERVICE is enabled and active"
}

preflight
if systemctl is-enabled --quiet "$SERVICE" 2> /dev/null && systemctl is-active --quiet "$SERVICE" 2> /dev/null; then
  ok "$SERVICE is already live, enabled and active; its last lines:"
  journalctl "_SYSTEMD_UNIT=$SERVICE.service" -n 5 -o cat --no-pager 2> /dev/null | sed 's/^/      /' || true
  exit 0
fi
first_pass
enable_service

say "== done"
if [ "$dry_run" -eq 1 ]; then
  say "dry run: nothing was changed"
  if [ "$blocked" -eq 1 ]; then
    failed=1
    exit 1
  fi
fi
