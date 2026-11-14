#!/usr/bin/env bash
# Provisions the sentinel's machine (deploy/SETUP.md sections 1, 3 and 6; deploy/PROVISION.md): the packages,
# unattended security upgrades with any reboot they need at a quiet hour, the time sync check; the `sentinel` user
# with rustup, Rust 1.99.1 and cargo-mutants 27.1.0; sentinel4rust built from this checkout's HEAD; a roster
# skeleton; GitHub's SSH host keys, checked against the fingerprints GitHub publishes; the two units and the polkit
# rule; then the isolation proof, deploy/verify-isolation.sh, and only if it passes `run_unit` in the roster.
# It does not enable the service: deploy/go-live.sh does, once repositories are onboarded.
#
# Whatever is already in place is left alone, so it can be re-run -- after a pull, it upgrades sentinel4rust to the
# new HEAD. It stops at the first failure.
#
#   sudo deploy/provision.sh [--dry-run] [--work-dir <dir>]
#
#   --dry-run         print every action without doing any; needs no root, but without it whatever the sentinel's
#                     home hides shows as still to do
#   --work-dir <dir>  the work_dir a new roster gets (default /home/sentinel/sentinel4rust); an existing roster keeps
#                     its own, and the units follow the roster's

set -euo pipefail

readonly SENTINEL_HOME=/home/sentinel
readonly DEFAULT_WORK_DIR=/home/sentinel/sentinel4rust
readonly TOOLCHAIN=1.99.1
readonly CARGO_MUTANTS_VERSION=27.1.0
# What rust-toolchain.toml names beside the compiler: there, rustup would otherwise fetch them on first use.
readonly COMPONENTS=(rustfmt clippy llvm-tools)
readonly REBOOT_TIME=04:30
readonly RUN_UNIT=sentinel4rust-mutants
readonly PACKAGES=(ca-certificates curl git build-essential pkg-config openssh-client polkitd unattended-upgrades)
# GitHub's SSH host key fingerprints as GitHub publishes them, at
# https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/githubs-ssh-key-fingerprints and in
# https://api.github.com/meta (ssh_key_fingerprints); both read on 2026-10-25. Update them only from those pages.
readonly GITHUB_FINGERPRINTS=(
  "SHA256:uNiVztksCsDhcc0u9e8BujQXVUpKZIDTMczCvj3tD2s"
  "SHA256:p2QAMXNIC1TJYWeIOttrVc98/R1BUFWu3/LiyKgUfQM"
  "SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU"
)

readonly cargo_bin="$SENTINEL_HOME/.cargo/bin"
readonly src_dir="$SENTINEL_HOME/src/sentinel4rust"
readonly installed_marker="$src_dir/INSTALLED_COMMIT"
readonly roster="$SENTINEL_HOME/.config/sentinel4rust/roster.toml"
readonly known_hosts="$SENTINEL_HOME/.ssh/known_hosts"
readonly units_dir=/etc/systemd/system
readonly polkit_rule=/etc/polkit-1/rules.d/50-sentinel4rust-mutants.rules
readonly apt_conf=/etc/apt/apt.conf.d/52sentinel4rust-unattended-upgrades
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly repo_root

dry_run=0
work_dir_option=""

say() { printf '%s\n' "$*"; }
ok() { say "ok    $*"; }
note() { say "note  $*"; }
warn() { say "WARN  $*"; }
failed=0
fail() {
  say "FAIL  $*" >&2
  failed=1
  exit 1
}

usage() {
  awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "${BASH_SOURCE[0]}"
  exit 2
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
    --work-dir) work_dir_option="${2:?--work-dir needs a directory}"; shift ;;
    --help | -h) usage ;;
    *) say "unknown argument $1" >&2; usage ;;
  esac
  shift
done

if [ "$(id -u)" -ne 0 ]; then
  [ "$dry_run" -eq 1 ] || fail "run it with sudo: sudo $0"
  note "a dry run as $(id -un), not root: whatever the sentinel's home hides from $(id -un) shows as to do"
fi
if [ -n "$work_dir_option" ]; then
  [[ "$work_dir_option" =~ ^/[A-Za-z0-9._/-]+$ ]] \
    || fail "--work-dir must be an absolute path of letters, digits, '.', '_', '-' and '/'"
  work_dir_option="${work_dir_option%/}"
fi

scratch="$(mktemp -d)"
finish() {
  local status=$?
  rm -rf "$scratch"
  if [ "$status" -ne 0 ] && [ "$failed" -eq 0 ]; then
    say "FAIL  stopped by the failing command above (exit $status): fix it and re-run; every step is idempotent" >&2
  fi
}
trap finish EXIT

# Runs a command as the sentinel, in a clean environment, in its home. Everything under its home is done this way,
# so root never follows a link the sentinel's code could have planted there.
as_sentinel() {
  local env=(env -i "--chdir=$SENTINEL_HOME" "HOME=$SENTINEL_HOME" USER=sentinel LOGNAME=sentinel LANG=C.UTF-8
    "PATH=$cargo_bin:/usr/local/bin:/usr/bin:/bin")
  if [ "$(id -u)" -eq 0 ]; then
    runuser -u sentinel -- "${env[@]}" "$@"
  else
    "${env[@]}" "$@"
  fi
}

# probe <command...>: a read-only check as the sentinel; false when there is no sentinel yet.
probe() {
  id sentinel > /dev/null 2>&1 && as_sentinel "$@" 2> /dev/null
}

git_head() {
  git -c safe.directory="$repo_root" -C "$repo_root" "$@"
}

# put_system_file <path> < content: root's file, written when missing or different.
put_system_file() {
  local path=$1
  cat > "$scratch/new"
  if [ -f "$path" ] && cmp -s "$scratch/new" "$path"; then
    ok "$path"
    return
  fi
  step "write $path" install -D -m 644 -o root -g root "$scratch/new" "$path"
  show_change "$path" "$scratch/new"
  system_changed=1
}

# show_change <old> <new>: in a dry run, the new file's text, or its difference from the old one.
show_change() {
  [ "$dry_run" -eq 1 ] || return 0
  if [ -r "$1" ]; then
    diff -u "$1" "$2" | tail -n +3 | sed 's/^/        /' || true
  else
    sed 's/^/        | /' "$2"
  fi
}

# sentinel_write <path> < content: the sentinel's file, replaced at once.
sentinel_write() {
  as_sentinel tee "$1.new" > /dev/null
  as_sentinel mv -f "$1.new" "$1"
}

roster_value() {
  local text
  text="$(probe cat "$roster" || true)"
  sed -n "s/^[[:space:]]*$1[[:space:]]*=[[:space:]]*\"\([^\"]*\)\".*/\1/p" <<< "$text" | sed -n 1p
}

preflight() {
  say "== preflight"
  local os_id="" pretty=""
  if [ -r /etc/os-release ]; then
    os_id="$(. /etc/os-release && printf '%s' "${ID:-}")"
    pretty="$(. /etc/os-release && printf '%s' "${PRETTY_NAME:-}")"
  fi
  [ "$os_id" = "ubuntu" ] || fail "this is for Ubuntu; this machine is '${pretty:-unknown}'"
  [ -d /run/systemd/system ] || fail "systemd is not running"
  head="$(git_head rev-parse HEAD)" || fail "$repo_root is not a git checkout"
  short="${head:0:7}"
  ok "$pretty, systemd; installing from $repo_root at $short"
  if [ -n "$(git_head --no-optional-locks status --porcelain)" ]; then
    warn "the checkout has uncommitted changes: only HEAD ($short) is installed"
  fi
}

packages() {
  say "== packages"
  local missing=() package status
  for package in "${PACKAGES[@]}"; do
    status="$(dpkg-query -W -f='${Status}' "$package" 2> /dev/null || true)"
    [ "$status" = "install ok installed" ] || missing+=("$package")
  done
  if [ "${#missing[@]}" -eq 0 ]; then
    ok "${PACKAGES[*]}"
  else
    step "apt-get update, then install ${missing[*]}" install_packages "${missing[@]}"
  fi
  if [ "$dry_run" -eq 0 ]; then
    [ -d /etc/polkit-1/rules.d ] || fail "polkit has no /etc/polkit-1/rules.d: the run unit's rule cannot be installed"
    systemctl is-active --quiet polkit || fail "polkit is not running: systemctl status polkit"
    ok "polkit is running"
  fi
}

install_packages() {
  DEBIAN_FRONTEND=noninteractive apt-get update -q
  DEBIAN_FRONTEND=noninteractive apt-get install -y -q "$@"
}

upgrades() {
  say "== unattended upgrades, rebooting at $REBOOT_TIME when one needs it"
  put_system_file "$apt_conf" << EOF
// Written by sentinel4rust's deploy/provision.sh. Security updates are installed unattended; when one needs a
// reboot, the machine reboots at a quiet hour, and the service starts again on boot. A run it interrupts is
// started again from the beginning.
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
Unattended-Upgrade::Automatic-Reboot "true";
Unattended-Upgrade::Automatic-Reboot-Time "$REBOOT_TIME";
EOF
  local timer
  for timer in apt-daily.timer apt-daily-upgrade.timer; do
    if systemctl is-enabled --quiet "$timer" 2> /dev/null; then
      ok "$timer is enabled"
    else
      step "enable $timer" systemctl enable --now "$timer"
    fi
  done
}

time_sync() {
  say "== time"
  local ntp synced
  ntp="$(timedatectl show -p NTP --value 2> /dev/null || printf unknown)"
  synced="$(timedatectl show -p NTPSynchronized --value 2> /dev/null || printf unknown)"
  if [ "$ntp" = "no" ]; then
    step "turn on network time synchronisation" timedatectl set-ntp true
  fi
  if [ "$synced" = "yes" ]; then
    ok "the clock is synchronised"
  else
    warn "the clock is not synchronised yet (NTP=$ntp, NTPSynchronized=$synced): the records' times and --status"
    warn "depend on it. Check again in a minute with timedatectl; PROVISION.md says when to stop"
  fi
}

sentinel_user() {
  say "== the sentinel user"
  if id sentinel > /dev/null 2>&1; then
    ok "user sentinel"
  else
    step "create the user sentinel, home $SENTINEL_HOME" useradd --create-home --home-dir "$SENTINEL_HOME" \
      --shell /bin/bash sentinel
  fi
  if id sentinel > /dev/null 2>&1; then
    local home
    home="$(getent passwd sentinel | cut -d: -f6)"
    [ "$home" = "$SENTINEL_HOME" ] || fail "sentinel's home is $home; the units expect $SENTINEL_HOME"
  fi
  local dir
  for dir in "$SENTINEL_HOME/.ssh" "$SENTINEL_HOME/.config/sentinel4rust" "$SENTINEL_HOME/src"; do
    if probe test -d "$dir"; then
      ok "$dir"
    else
      step "create $dir" as_sentinel mkdir -p "$dir"
    fi
  done
  if [ "$dry_run" -eq 0 ]; then
    as_sentinel chmod 700 "$SENTINEL_HOME/.ssh"
  fi
}

toolchain_installed() {
  local listed
  listed="$(probe "$cargo_bin/rustup" toolchain list)" || return 1
  grep -q "^$TOOLCHAIN-" <<< "$listed"
}

components_installed() {
  local installed component
  installed="$(probe "$cargo_bin/rustup" component list --installed --toolchain "$TOOLCHAIN")" || return 1
  for component in "${COMPONENTS[@]}"; do
    grep -q "^$component-" <<< "$installed" || return 1
  done
}

rust() {
  say "== rustup, Rust $TOOLCHAIN, cargo-mutants $CARGO_MUTANTS_VERSION"
  if probe test -x "$cargo_bin/rustup"; then
    ok "$(probe "$cargo_bin/rustup" --version | sed -n 1p)"
  else
    step "install rustup for sentinel from https://sh.rustup.rs, default toolchain $TOOLCHAIN" install_rustup
  fi
  if ! toolchain_installed; then
    step "install Rust $TOOLCHAIN" as_sentinel rustup toolchain install "$TOOLCHAIN" --profile minimal
  fi
  if components_installed; then
    ok "Rust $TOOLCHAIN with ${COMPONENTS[*]}"
  else
    step "add ${COMPONENTS[*]} to Rust $TOOLCHAIN" as_sentinel rustup component add --toolchain "$TOOLCHAIN" \
      "${COMPONENTS[@]}"
  fi
  if [[ "$(probe "$cargo_bin/rustup" default || true)" == "$TOOLCHAIN-"* ]]; then
    ok "the default toolchain is $TOOLCHAIN"
  else
    step "make $TOOLCHAIN the default toolchain" as_sentinel rustup default "$TOOLCHAIN"
  fi
  if [ "$(probe cargo mutants --version || true)" = "cargo-mutants $CARGO_MUTANTS_VERSION" ]; then
    ok "cargo-mutants $CARGO_MUTANTS_VERSION"
  else
    step "install cargo-mutants $CARGO_MUTANTS_VERSION (--locked)" as_sentinel cargo install cargo-mutants \
      --version "$CARGO_MUTANTS_VERSION" --locked
  fi
}

install_rustup() {
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | as_sentinel sh -s -- -y --profile minimal --default-toolchain "$TOOLCHAIN"
}

sentinel4rust_binary() {
  say "== sentinel4rust"
  if [ "$(probe cat "$installed_marker" || true)" = "$head" ] && probe test -x "$cargo_bin/sentinel4rust"; then
    ok "$(probe "$cargo_bin/sentinel4rust" --version), built from $short"
    return
  fi
  step "build sentinel4rust from $short: git archive into $src_dir, cargo install --path cli --locked" \
    install_sentinel4rust
  sentinel4rust_installed=1
}

install_sentinel4rust() {
  as_sentinel rm -rf "$src_dir"
  as_sentinel mkdir -p "$src_dir"
  git_head archive --format=tar HEAD | as_sentinel tar -x -C "$src_dir"
  as_sentinel env --chdir="$src_dir" cargo install --path cli --locked --force
  printf '%s\n' "$head" | sentinel_write "$installed_marker"
  ok "$(as_sentinel "$cargo_bin/sentinel4rust" --version), built from $short"
}

roster_skeleton() {
  say "== the roster"
  if probe test -f "$roster"; then
    work_dir="$(roster_value work_dir)"
    [ -n "$work_dir" ] || fail "$roster has no work_dir"
    ok "$roster is there and is left as it is (work_dir $work_dir)"
    if [ -n "$work_dir_option" ] && [ "$work_dir_option" != "$work_dir" ]; then
      warn "--work-dir $work_dir_option is ignored: the roster keeps $work_dir"
    fi
  else
    work_dir="${work_dir_option:-$DEFAULT_WORK_DIR}"
    roster_text > "$scratch/skeleton"
    step "write the roster skeleton $roster (work_dir $work_dir)" sentinel_write "$roster" < "$scratch/skeleton"
    show_change "" "$scratch/skeleton"
  fi
  [[ "$work_dir" =~ ^/[A-Za-z0-9._/-]+$ ]] || fail "work_dir $work_dir must be an absolute path the units can name"
  if probe test -d "$work_dir"; then
    ok "work_dir $work_dir"
  elif [[ "$work_dir" == "$SENTINEL_HOME"/* ]]; then
    step "create the work_dir $work_dir" as_sentinel mkdir -p "$work_dir"
  else
    step "create the work_dir $work_dir, owned by sentinel" install -d -o sentinel -g sentinel -m 755 "$work_dir"
  fi
}

roster_text() {
  cat << EOF
# The sentinel's roster. deploy/provision.sh wrote this skeleton on $(date -u +%Y-%m-%d) and never overwrites it.
# deploy/onboard.sh sets results_url and appends the repositories; the service needs both before it starts. Every key, with its default,
# is in docs/ARCHITECTURE.md, and deploy/roster.example.toml is an annotated example. Edit, then, once no run is in
# progress: sudo systemctl restart sentinel4rust
#
# Every watched repository's results go to its folder of the private results repository (deploy/SETUP.md
# section 7).

work_dir = "$work_dir"
pause_seconds = 300
jobs = 2
run_timeout_hours = 12
config_version = 1
stale_after_hours = 24
EOF
}

is_published() {
  local fingerprint
  for fingerprint in "${GITHUB_FINGERPRINTS[@]}"; do
    [ "$1" = "$fingerprint" ] && return 0
  done
  return 1
}

known_hosts() {
  say "== GitHub's SSH host keys"
  local scan line fingerprint material count=0 added=0
  scan="$(ssh-keyscan -T 15 -t rsa,ecdsa,ed25519 github.com 2> /dev/null | grep -v '^#' || true)"
  [ -n "$scan" ] || fail "ssh-keyscan github.com returned no key: is the network up?"
  while IFS= read -r line; do
    fingerprint="$(printf '%s\n' "$line" | ssh-keygen -lf - | awk '{ print $2 }')"
    is_published "$fingerprint" \
      || fail "github.com offered a key with fingerprint $fingerprint, which GitHub does not publish: refusing"
    count=$((count + 1))
  done <<< "$scan"
  ok "github.com offered $count keys, each with a fingerprint GitHub publishes"
  if probe test -f "$known_hosts"; then
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      fingerprint="$(printf '%s\n' "$line" | ssh-keygen -lf - | awk '{ print $2 }')"
      is_published "$fingerprint" || fail "$known_hosts holds a github.com key GitHub does not publish
      ($fingerprint): remove it with ssh-keygen -R github.com -f $known_hosts, as sentinel, then re-run"
    done < <(probe ssh-keygen -F github.com -f "$known_hosts" | grep -v '^#' || true)
  fi
  : > "$scratch/hosts"
  while IFS= read -r line; do
    material="$(awk '{ print $2 " " $3 }' <<< "$line")"
    if ! probe grep -qF -- "$material" "$known_hosts"; then
      printf '%s\n' "$line" >> "$scratch/hosts"
      added=$((added + 1))
    fi
  done <<< "$scan"
  if [ "$added" -eq 0 ]; then
    ok "$known_hosts holds them"
  else
    step "add $added verified github.com keys to $known_hosts" append_known_hosts
  fi
}

append_known_hosts() {
  as_sentinel tee -a "$known_hosts" < "$scratch/hosts" > /dev/null
  as_sentinel chmod 644 "$known_hosts"
}

units() {
  say "== the units and the polkit rule"
  system_changed=0
  git_head show HEAD:deploy/sentinel4rust.service > "$scratch/service"
  git_head show "HEAD:deploy/$RUN_UNIT@.service" > "$scratch/run-unit"
  git_head show HEAD:deploy/50-sentinel4rust-mutants.rules > "$scratch/rule"
  put_system_file "$units_dir/sentinel4rust.service" < "$scratch/service"
  put_system_file "$units_dir/$RUN_UNIT@.service" < "$scratch/run-unit"
  put_system_file "$polkit_rule" < "$scratch/rule"
  local service_dropin="$units_dir/sentinel4rust.service.d/work-dir.conf"
  local run_dropin="$units_dir/$RUN_UNIT@.service.d/work-dir.conf"
  if [ "$work_dir" = "$DEFAULT_WORK_DIR" ]; then
    local dropin
    for dropin in "$service_dropin" "$run_dropin"; do
      if [ -e "$dropin" ]; then
        step "remove $dropin: the work_dir is the units' own again" rm -f "$dropin"
        system_changed=1
      fi
    done
  else
    printf '[Service]\nReadWritePaths=%s\nEnvironment=SENTINEL4RUST_LOCK=%s/sentinel4rust.lock\n' \
      "$work_dir" "$work_dir" > "$scratch/service-dropin"
    printf '[Service]\nExecStart=\nExecStart=%s/sentinel4rust --run-job %s/jobs/%%i\nReadWritePaths=%s\n' \
      "$cargo_bin" "$work_dir" "$work_dir" > "$scratch/run-dropin"
    put_system_file "$service_dropin" < "$scratch/service-dropin"
    put_system_file "$run_dropin" < "$scratch/run-dropin"
  fi
  if [ "$system_changed" -eq 1 ]; then
    step "reload systemd" systemctl daemon-reload
  fi
}

isolation() {
  say "== the run unit, away from the keys"
  local unit
  unit="$(roster_value run_unit)"
  if systemctl is-active --quiet sentinel4rust 2> /dev/null; then
    note "the service is running, and the proof shares $work_dir/jobs/run with it: not re-run now"
    if [ "$unit" = "$RUN_UNIT" ]; then
      ok "run_unit = \"$RUN_UNIT\" in the roster"
    else
      warn "run_unit is not \"$RUN_UNIT\": stop the service and re-run provision.sh to prove and set it"
    fi
    return
  fi
  if [ "$dry_run" -eq 1 ]; then
    say "would prove the run unit cannot read $SENTINEL_HOME/.ssh: $src_dir/deploy/verify-isolation.sh $work_dir"
  else
    say "do    prove the run unit cannot read $SENTINEL_HOME/.ssh: $src_dir/deploy/verify-isolation.sh $work_dir"
    if ! as_sentinel "$src_dir/deploy/verify-isolation.sh" "$work_dir" "$RUN_UNIT" | sed 's/^/      /'; then
      fail "the isolation proof failed, so run_unit is not set: cargo-mutants would run beside the keys"
    fi
  fi
  if [ "$unit" = "$RUN_UNIT" ]; then
    ok "run_unit = \"$RUN_UNIT\" in the roster"
  elif [ -n "$unit" ]; then
    warn "the roster names run_unit \"$unit\", not \"$RUN_UNIT\": left as it is"
  else
    if ! probe cat "$roster" > "$scratch/roster.old"; then
      [ "$dry_run" -eq 1 ] || fail "cannot read $roster"
      cat "$scratch/skeleton" > "$scratch/roster.old"
    fi
    run_unit_roster < "$scratch/roster.old" > "$scratch/roster.new"
    step "set run_unit = \"$RUN_UNIT\" in $roster, before its first [[repo]]" \
      sentinel_write "$roster" < "$scratch/roster.new"
    show_change "$scratch/roster.old" "$scratch/roster.new"
  fi
}

# The roster on stdin, with run_unit before its first [[repo]]: a key after one would belong to that repository.
run_unit_roster() {
  local line
  line="run_unit = \"$RUN_UNIT\"   # proven by deploy/verify-isolation.sh on $(date -u +%Y-%m-%d)"
  awk -v line="$line" '
    !done && /^[ \t]*\[\[repo\]\]/ { print line; print ""; done = 1 }
    { print }
    END { if (!done) print line }'
}

sentinel4rust_installed=0
preflight
packages
upgrades
time_sync
sentinel_user
rust
sentinel4rust_binary
roster_skeleton
known_hosts
units
isolation

say "== done"
if [ "$dry_run" -eq 1 ]; then
  say "dry run: nothing was changed"
fi
if systemctl is-enabled --quiet sentinel4rust 2> /dev/null; then
  if [ "$sentinel4rust_installed" -eq 1 ]; then
    note "the service runs the binary it started with. Once no run is in progress (systemctl is-active"
    note "$RUN_UNIT@run.service says inactive), restart it: sudo systemctl restart sentinel4rust"
  fi
else
  say "The service is not enabled. Next: deploy/onboard.sh, then deploy/go-live.sh (deploy/PROVISION.md)."
fi
