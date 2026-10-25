#!/bin/sh
# Proves the run unit cannot read the deploy keys (deploy/SETUP.md section 6). Run as the `sentinel` user, after
# installing the unit and the polkit rule: it starts the unit as the service does, on a job listing ~/.ssh, and
# passes only when the listing is refused while the keys stay readable outside the unit, and when the unit, started
# on a job that waits, can be stopped as the service stops it.
#
#   deploy/verify-isolation.sh <work_dir> [unit]       e.g. /home/sentinel/sentinel4rust sentinel4rust-mutants

set -u
work_dir="${1:?usage: verify-isolation.sh <work_dir> [unit]}"
unit="${2:-sentinel4rust-mutants}"
job="$work_dir/jobs/run"

echo "== outside the unit"
if [ -r "$HOME/.ssh" ]; then
  echo "ok    $HOME/.ssh is readable here, where git runs"
else
  echo "FAIL  $HOME/.ssh is not readable even outside the unit"; exit 1
fi

rm -rf "$job" && mkdir -p "$job"
cat > "$job/job.toml" <<TOML
program = "sh"
dir = "$job"
timeout_seconds = 60

[[arg]]
value = "-c"

[[arg]]
value = "ls -la \"\$HOME/.ssh\""
TOML

echo "== inside $unit@run.service"
if ! systemctl start --wait "$unit@run.service"; then
  echo "FAIL  could not start $unit@run.service: install the unit and the polkit rule"; exit 1
fi
if grep -q 'code = 0' "$job/result.toml" 2>/dev/null; then
  echo "FAIL  the unit listed $HOME/.ssh:"; cat "$job/stdout"; exit 1
fi
if grep -qi 'permission denied' "$job/stderr" 2>/dev/null; then
  echo "ok    $HOME/.ssh is unreadable inside the unit"
else
  echo "FAIL  the job did not report a refused listing:"; cat "$job/result.toml" "$job/stderr" 2>&1; exit 1
fi

# The service stops a run when it stops or restarts (deploy/sentinel4rust.service, docs/OPEN_POINTS.md point 19).
echo "== stopping $unit@run.service, as the service does"
rm -rf "$job" && mkdir -p "$job"
cat > "$job/job.toml" <<TOML
program = "sleep"
dir = "$job"
timeout_seconds = 120

[[arg]]
value = "120"
TOML
systemctl start --no-block "$unit@run.service"
waited=0
while [ "$(systemctl is-active "$unit@run.service")" != "activating" ] && [ "$waited" -lt 30 ]; do
  sleep 1; waited=$((waited + 1))
done
if [ "$(systemctl is-active "$unit@run.service")" != "activating" ]; then
  echo "FAIL  $unit@run.service did not start on a job that waits"; exit 1
fi
if ! systemctl stop "$unit@run.service"; then
  echo "FAIL  could not stop $unit@run.service: the polkit rule must allow stop, or a restart leaves a run behind"
  exit 1
fi
state="$(systemctl is-active "$unit@run.service")"
case "$state" in
  inactive | failed) echo "ok    $unit@run.service stopped ($state)" ;;
  *) echo "FAIL  $unit@run.service is $state after the stop"; exit 1 ;;
esac
echo "RESULT: ok"
