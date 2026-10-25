#!/usr/bin/env bash
# Onboards repositories onto the sentinel (deploy/SETUP.md section 2, deploy/PROVISION.md). Per repository: a deploy
# key and an SSH host alias for the `sentinel` user; on GitHub, that key with write access and the ruleset in
# deploy/rulesets/sentinel-only-mutants.json; the access proof, deploy/verify-access.sh; then the repository's
# [[repo]] entry in the roster, with the date the access was verified. Whatever is already in place is left alone,
# so it can be re-run; it stops at the first failure.
#
# Run as the owner, from the checkout, after deploy/provision.sh, with sudo cached (sudo -v) and gh logged in.
#
#   deploy/onboard.sh [options] <repo>...
#
#   --dry-run              print every action without doing any: GitHub is read, never written
#   --apply-github         allow the GitHub changes, a deploy key and a ruleset per repository -- passed only after
#                          the owner's go (deploy/PROVISION.md); without it, onboard.sh refuses to change anything
#   --github-owner <name>  the repositories' owner (default umbgtt10)
#   --roster <file>        the roster (default ~sentinel/.config/sentinel4rust/roster.toml)
#   --key-title <title>    the deploy key's title on GitHub (default sentinel4rust)
#   --local <dir>          test mode: <dir>/<repo>.git are bare repositories, reached through file:// URLs; the
#                          sentinel's side is the current user's, under <dir>/sentinel-home; no sudo, no GitHub
#                          and no access proof

set -euo pipefail

readonly RULESET_NAME="sentinel: only mutants"
deploy_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly deploy_dir
readonly ruleset_file="$deploy_dir/rulesets/sentinel-only-mutants.json"

dry_run=0
apply_github=0
owner="umbgtt10"
roster=""
key_title="sentinel4rust"
local_dir=""
repos=()

usage() {
  awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "${BASH_SOURCE[0]}"
  exit 2
}

say() { printf '%s\n' "$*"; }
ok() { say "ok    $*"; }
note() { say "note  $*"; }
failed=0
fail() {
  say "FAIL  $*" >&2
  failed=1
  exit 1
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
    --apply-github) apply_github=1 ;;
    --github-owner) owner="${2:?--github-owner needs a name}"; shift ;;
    --roster) roster="${2:?--roster needs a file}"; shift ;;
    --key-title) key_title="${2:?--key-title needs a title}"; shift ;;
    --local) local_dir="${2:?--local needs a directory}"; shift ;;
    --help | -h) usage ;;
    -*) say "unknown option $1" >&2; usage ;;
    *) repos+=("$1") ;;
  esac
  shift
done

[ "${#repos[@]}" -gt 0 ] || usage
for repo in "${repos[@]}"; do
  [[ "$repo" =~ ^[A-Za-z0-9_-][A-Za-z0-9._-]*$ ]] \
    || fail "invalid repository name '$repo' (letters, digits, '-', '_', '.', not starting with '.')"
done
[[ "$owner" =~ ^[A-Za-z0-9][A-Za-z0-9-]*$ ]] || fail "invalid owner '$owner'"

if [ -n "$local_dir" ]; then
  [ "$apply_github" -eq 0 ] || fail "--local never touches GitHub: drop --apply-github"
  [ -d "$local_dir" ] || fail "no directory $local_dir"
  local_dir="$(cd "$local_dir" && pwd)"
  sentinel_home="$local_dir/sentinel-home"
  sentinel_exists=1
else
  if [ "$dry_run" -eq 0 ] && [ "$apply_github" -eq 0 ]; then
    fail "refusing: onboarding changes GitHub -- a deploy key and a ruleset per repository. Pass --apply-github,
      and only after the owner's go in the chat (deploy/PROVISION.md); --dry-run prints the plan"
  fi
  command -v gh >/dev/null || fail "gh is not installed"
  gh auth status >/dev/null 2>&1 || fail "gh is not logged in: gh auth login"
  [ -f "$ruleset_file" ] || fail "no $ruleset_file"
  grep -qF "\"name\": \"$RULESET_NAME\"" "$ruleset_file" || fail "$ruleset_file is not named '$RULESET_NAME'"
  if id sentinel >/dev/null 2>&1; then
    sentinel_exists=1
    sentinel_home="$(getent passwd sentinel | cut -d: -f6)"
    sudo -n true 2>/dev/null || fail "sudo is not cached: run sudo -v, then re-run"
  else
    [ "$dry_run" -eq 1 ] || fail "there is no sentinel user: run deploy/provision.sh first"
    sentinel_exists=0
    sentinel_home="/home/sentinel"
    note "there is no sentinel user yet: everything on its side would be created"
  fi
fi
readonly ssh_dir="$sentinel_home/.ssh"
readonly ssh_config="$ssh_dir/config"
[ -n "$roster" ] || roster="$sentinel_home/.config/sentinel4rust/roster.toml"
readonly verify_access="$sentinel_home/src/sentinel4rust/deploy/verify-access.sh"

scratch="$(mktemp -d)"
finish() {
  local status=$?
  rm -rf "$scratch"
  if [ "$status" -ne 0 ] && [ "$failed" -eq 0 ]; then
    say "FAIL  stopped by the failing command above (exit $status): fix it and re-run; every step is idempotent" >&2
  fi
}
trap finish EXIT

# Runs a command as the sentinel user, in its home (the owner's checkout may be unreadable to it).
as_sentinel() {
  if [ -n "$local_dir" ]; then
    "$@"
  else
    sudo -n -u sentinel -H env --chdir="$sentinel_home" "$@"
  fi
}

# Whether the sentinel has a file; false when there is no sentinel yet.
sentinel_has() {
  [ "$sentinel_exists" -eq 1 ] && as_sentinel test -e "$1"
}

ensure_home() {
  if [ -n "$local_dir" ] && [ ! -d "$ssh_dir" ]; then
    step "create the stand-in sentinel home $sentinel_home" mkdir -p -m 700 "$ssh_dir"
  fi
  if sentinel_has "$roster"; then
    ok "roster $roster"
  elif [ "$sentinel_exists" -eq 1 ]; then
    fail "no roster at $roster: run deploy/provision.sh first, or pass --roster"
  fi
  if [ -z "$local_dir" ] && [ "$sentinel_exists" -eq 1 ] && ! sentinel_has "$verify_access"; then
    fail "no $verify_access: run deploy/provision.sh first"
  fi
}

ensure_key() {
  local repo=$1 key="$ssh_dir/sentinel_$1"
  if sentinel_has "$key.pub" && sentinel_has "$key"; then
    ok "key $key: $(as_sentinel ssh-keygen -lf "$key.pub")"
  elif sentinel_has "$key"; then
    fail "$key exists without $key.pub: restore it with ssh-keygen -y, or remove both and re-run"
  else
    step "create the key $key (ed25519, no passphrase)" \
      as_sentinel ssh-keygen -q -t ed25519 -N "" -C "sentinel4rust $repo" -f "$key"
    [ "$dry_run" -eq 1 ] || ok "key $key: $(as_sentinel ssh-keygen -lf "$key.pub")"
  fi
}

ensure_host_alias() {
  local repo=$1
  if sentinel_has "$ssh_config" && as_sentinel grep -qxF "Host github-$repo" "$ssh_config"; then
    ok "host alias github-$repo in $ssh_config"
    return
  fi
  cat > "$scratch/alias" << EOF

Host github-$repo
    HostName github.com
    User git
    IdentityFile $ssh_dir/sentinel_$repo
    IdentitiesOnly yes
    StrictHostKeyChecking yes
EOF
  step "add the host alias github-$repo to $ssh_config" append_as_sentinel "$scratch/alias" "$ssh_config"
  [ "$dry_run" -eq 1 ] || as_sentinel chmod 600 "$ssh_config"
}

# append_as_sentinel <source> <file>: appends a file the owner wrote to one the sentinel owns.
append_as_sentinel() {
  as_sentinel tee -a "$2" < "$1" > /dev/null
}

check_local_repo() {
  local repo=$1 bare="$local_dir/$1.git"
  [ "$(git --git-dir="$bare" rev-parse --is-bare-repository 2>/dev/null)" = "true" ] \
    || fail "$bare is not a bare repository"
  ok "bare repository $bare"
  git --git-dir="$bare" cat-file -e main:.cargo/mutants.toml 2>/dev/null \
    || note "$repo has no .cargo/mutants.toml on main: the sentinel skips it until it has one"
}

check_github_repo() {
  local repo=$1 info visibility permission
  info="$(gh repo view "$owner/$repo" --json visibility,viewerPermission \
    --jq '.visibility + " " + .viewerPermission')" || fail "cannot read $owner/$repo on GitHub"
  visibility="${info% *}"
  permission="${info#* }"
  [ "$permission" = "ADMIN" ] || fail "$owner/$repo: deploy keys and rulesets need admin, gh has $permission"
  ok "$owner/$repo is $visibility, and gh is its admin"
  case "$visibility" in
    PUBLIC) note "its mutants branch will be public, survivors and log tails included (deploy/SETUP.md section 7)" ;;
    *) note "rulesets on a $visibility repository need a paid plan (GitHub Pro or above)" ;;
  esac
  gh api "repos/$owner/$repo/contents/.cargo/mutants.toml?ref=main" --silent 2>/dev/null \
    || note "$repo has no .cargo/mutants.toml on main: the sentinel skips it until it has one"
}

ensure_deploy_key() {
  local repo=$1 key="$ssh_dir/sentinel_$1" listing ours="" title key_text read_only
  listing="$(gh repo deploy-key list -R "$owner/$repo" --json title,key,readOnly \
    --jq '.[] | [.title, .key, (.readOnly | tostring)] | @tsv')" \
    || fail "cannot list $owner/$repo's deploy keys"
  if sentinel_has "$key.pub"; then
    ours="$(as_sentinel cat "$key.pub" | awk '{ print $1 " " $2 }')"
  fi
  while IFS=$'\t' read -r title key_text read_only; do
    [ -n "$title" ] || continue
    if [ -n "$ours" ] && [ "$(printf '%s' "$key_text" | awk '{ print $1 " " $2 }')" = "$ours" ]; then
      [ "$read_only" = "false" ] \
        || fail "$owner/$repo has this key read-only, titled '$title': delete it there (Settings, Deploy keys), re-run"
      ok "deploy key '$title' on $owner/$repo, with write access"
      return
    fi
    [ "$title" != "$key_title" ] \
      || fail "$owner/$repo has a deploy key titled '$key_title' that is not $key.pub -- perhaps a previous
      machine's. If it is, delete it there (Settings, Deploy keys) and re-run; the owner decides"
  done <<< "$listing"
  if [ "$dry_run" -eq 0 ]; then
    as_sentinel cat "$key.pub" > "$scratch/$repo.pub"
  fi
  step "add $key.pub to $owner/$repo as the deploy key '$key_title', with write access" \
    gh repo deploy-key add "$scratch/$repo.pub" -R "$owner/$repo" --title "$key_title" --allow-write
}

# Sets `blocked` when GitHub offers no rulesets on the repository: the access proof could never pass there.
ensure_ruleset() {
  local repo=$1 found id
  blocked=0
  if ! found="$(gh api "repos/$owner/$repo/rulesets" --paginate \
    --jq ".[] | select(.name == \"$RULESET_NAME\") | \"\(.id) \(.enforcement)\"" 2> "$scratch/error")"; then
    sed 's/^/      /' "$scratch/error" >&2
    grep -q 'HTTP 403' "$scratch/error" || fail "cannot list $owner/$repo's rulesets"
    say "BLOCKED $owner/$repo offers no rulesets (GitHub's answer is above), so nothing could keep its deploy key off"
    say "        main. Nothing was changed on GitHub for it. Ask the owner: make it public, move to a paid plan, or"
    say "        leave it off the roster (docs/ADRs/ADR-DeployKeyAndRuleset.md)"
    [ "$dry_run" -eq 1 ] || fail "$owner/$repo cannot be onboarded: stop, and ask the owner"
    blocked=1
    return
  fi
  found="${found%%$'\n'*}"
  if [ -n "$found" ]; then
    ok "ruleset '$RULESET_NAME' on $owner/$repo (id ${found% *}, ${found#* })"
    [ "${found#* }" = "active" ] || note "it is not active: the access proof will fail until it is"
    return
  fi
  if [ "$dry_run" -eq 1 ]; then
    say "would create the ruleset '$RULESET_NAME' on $owner/$repo from $ruleset_file"
    return
  fi
  say "do    create the ruleset '$RULESET_NAME' on $owner/$repo from $ruleset_file"
  id="$(gh api --method POST "repos/$owner/$repo/rulesets" --input "$ruleset_file" --jq '.id')" \
    || fail "GitHub refused the ruleset on $owner/$repo (its message is above). A private repository needs a paid
      plan for one (docs/ADRs/ADR-DeployKeyAndRuleset.md): stop, and ask the owner"
  ok "ruleset '$RULESET_NAME' on $owner/$repo (id $id)"
}

prove_access() {
  local repo=$1 url=$2
  if [ "$dry_run" -eq 1 ]; then
    say "would prove the access as sentinel: $verify_access $url"
    return
  fi
  say "do    prove the access as sentinel: $verify_access $url"
  if ! as_sentinel "$verify_access" "$url" | sed 's/^/      /'; then
    fail "the access proof failed for $repo: it is not added to the roster"
  fi
}

has_entry() {
  as_sentinel cat "$roster" | awk -v name="$1" '
    { line = $0; sub(/#.*/, "", line) }
    line ~ /^[ \t]*name[ \t]*=/ {
      value = line; sub(/^[ \t]*name[ \t]*=[ \t]*"/, "", value); sub(/".*$/, "", value)
      if (value == name) found = 1
    }
    END { exit found ? 0 : 1 }'
}

ensure_entry() {
  local repo=$1 url=$2 today proof
  if sentinel_has "$roster" && has_entry "$repo"; then
    ok "$repo is on the roster"
    return
  fi
  today="$(date -u +%Y-%m-%d)"
  if [ -n "$local_dir" ]; then
    proof="# onboarded $today by deploy/onboard.sh --local: a file:// test repository, no access proof"
  else
    proof="# access verified: $today (deploy/verify-access.sh, by deploy/onboard.sh)"
  fi
  printf '\n[[repo]]\nname = "%s"\nurl = "%s"\n%s\n' "$repo" "$url" "$proof" > "$scratch/entry"
  step "append [[repo]] $repo ($url) to $roster" append_as_sentinel "$scratch/entry" "$roster"
  appended=$((appended + 1))
}

# The ruleset goes on before the deploy key, so that the key is never on GitHub without it.
onboard() {
  local repo=$1 url
  say "== $repo"
  if [ -n "$local_dir" ]; then
    url="file://$local_dir/$repo.git"
    ensure_key "$repo"
    ensure_host_alias "$repo"
    check_local_repo "$repo"
  else
    url="git@github-$repo:$owner/$repo.git"
    check_github_repo "$repo"
    ensure_ruleset "$repo"
    if [ "$blocked" -eq 1 ]; then
      blocked_repos+=("$repo")
      return
    fi
    ensure_key "$repo"
    ensure_host_alias "$repo"
    ensure_deploy_key "$repo"
    prove_access "$repo" "$url"
  fi
  ensure_entry "$repo" "$url"
}

appended=0
blocked=0
blocked_repos=()
say "== the sentinel's side, under $sentinel_home"
ensure_home
for repo in "${repos[@]}"; do
  onboard "$repo"
done

say "== done"
if [ "$dry_run" -eq 1 ]; then
  say "dry run: nothing was changed"
  if [ "${#blocked_repos[@]}" -gt 0 ]; then
    say "BLOCKED ${blocked_repos[*]}: cannot be onboarded as they are (see above)"
    failed=1
    exit 1
  fi
elif [ "$appended" -gt 0 ] && [ -z "$local_dir" ] && systemctl is-active --quiet sentinel4rust; then
  note "the service reads the roster on start. Once no run is in progress -- systemctl is-active"
  note "sentinel4rust-mutants@run.service says inactive, or journalctl -u sentinel4rust shows a visit just"
  note "ended -- restart it: sudo systemctl restart sentinel4rust"
fi
