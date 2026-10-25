#!/usr/bin/env bash
# Watches the sentinel from another machine, such as the owner's laptop (deploy/SETUP.md section 5,
# deploy/PROVISION.md): one read-only `sentinel4rust --status` pass over a roster that reaches the repositories with
# this machine's own access -- https, or the owner's SSH key; never the sentinel's keys. Quiet while every
# repository's record is current or has waited less than stale_after_hours; otherwise it prints the status lines,
# mails them with --mail, and exits with --status's code: 1 when one needs attention, 2 for a bad command line.
#
#   deploy/status-check.sh --roster <file> [--mail <address>] [--sentinel <binary>] [--verbose] [--dry-run]
#
#   --roster <file>      the roster --status reads; its work_dir keeps the clones, under <work_dir>/status
#   --mail <address>     also mail the lines on failure, with mail(1) (mailutils or bsd-mailx)
#   --sentinel <binary>  the sentinel4rust to run (default: the one on PATH)
#   --verbose            print the lines when nothing needs attention too
#   --dry-run            print what it would run, and run nothing
#
# Every six hours from cron, which mails what a job prints when MAILTO is set:
#   0 */6 * * * $HOME/sentinel4rust/deploy/status-check.sh --roster $HOME/.config/sentinel4rust/status.toml
# or from a systemd user timer: deploy/PROVISION.md, "Watching it from another machine".

set -euo pipefail

roster=""
mail_to=""
sentinel="sentinel4rust"
verbose=0
dry_run=0

usage() {
  awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "${BASH_SOURCE[0]}"
  exit 2
}

while [ $# -gt 0 ]; do
  case "$1" in
    --roster) roster="${2:?--roster needs a file}"; shift ;;
    --mail) mail_to="${2:?--mail needs an address}"; shift ;;
    --sentinel) sentinel="${2:?--sentinel needs a binary}"; shift ;;
    --verbose) verbose=1 ;;
    --dry-run) dry_run=1 ;;
    --help | -h) usage ;;
    *) printf 'unknown argument %s\n' "$1" >&2; usage ;;
  esac
  shift
done
[ -n "$roster" ] || usage
refuse() {
  printf 'FAIL  %s\n' "$*" >&2
  exit 2
}
[ -f "$roster" ] || refuse "no roster at $roster"
command -v "$sentinel" > /dev/null \
  || refuse "no $sentinel: install it from a checkout, cargo install --path core --locked"

if [ "$dry_run" -eq 1 ]; then
  printf 'would run %s --roster %s --status\n' "$sentinel" "$roster"
  if [ -n "$mail_to" ]; then
    printf 'would mail its lines to %s if it exits non-zero\n' "$mail_to"
  fi
  exit 0
fi

status=0
lines="$("$sentinel" --roster "$roster" --status 2>&1)" || status=$?
if [ "$status" -eq 0 ]; then
  [ "$verbose" -eq 0 ] || printf '%s\n' "$lines"
  exit 0
fi

checked="checked from $(hostname) at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
report="sentinel4rust --status needs attention (exit $status), $checked:
$lines"
printf '%s\n' "$report"
if [ -n "$mail_to" ]; then
  if command -v mail > /dev/null; then
    printf '%s\n' "$report" | mail -s "sentinel4rust needs attention" "$mail_to"
  else
    printf 'note  no mail command (mailutils or bsd-mailx): the lines are printed only\n' >&2
  fi
fi
exit "$status"
