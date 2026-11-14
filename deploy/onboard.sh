#!/usr/bin/env bash
# Onboards repositories onto the sentinel (deploy/SETUP.md section 2, deploy/PROVISION.md,
# docs/ADRs/ADR-SharedResultsRepository.md). Once: the results repository -- a deploy key and an SSH host alias for
# the `sentinel` user, that key on GitHub with write access to the results repository and nothing else, and
# `results_url` in the roster. Per watched repository: that it is public, so the sentinel reads it over https with
# no credential at all; that no deploy key titled like the sentinel's is left on it; then its [[repo]] entry in the
# roster. The access proof, deploy/verify-access.sh, runs before anything is added to the roster. Whatever is
# already in place is left alone, so it can be re-run; it stops at the first failure.
#
# Run as the owner, from the checkout, after deploy/provision.sh, with sudo cached (sudo -v) and gh logged in.
#
#   deploy/onboard.sh [options] <repo>...
#
#   --dry-run              print every action without doing any: GitHub is read, never written
#   --apply-github         allow the one GitHub change, the deploy key on the results repository -- passed only after
#                          the owner's go (deploy/PROVISION.md); without it, onboard.sh refuses to change anything
#   --github-owner <name>  the repositories' owner (default umbgtt10)
#   --results <name>       the results repository, under the same owner (default mutants)
#   --roster <file>        the roster (default ~sentinel/.config/sentinel4rust/roster.toml)
#   --key-title <title>    the deploy key's title on GitHub (default sentinel4rust)
#   --local <dir>          test mode: <dir>/<repo>.git and <dir>/<results>.git are bare repositories, reached through
#                          file:// URLs; the sentinel's side is the current user's, under <dir>/sentinel-home; no
#                          sudo, no GitHub and no access proof

set -euo pipefail

deploy_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly deploy_dir

dry_run=0
apply_github=0
owner="umbgtt10"
results="mutants"
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
    --results) results="${2:?--results needs a name}"; shift ;;
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
for repo in "$results" "${repos[@]}"; do
  [[ "$repo" =~ ^[A-Za-z0-9_-][A-Za-z0-9._-]*$ ]] \
    || fail "invalid repository name '$repo' (letters, digits, '-', '_', '.', not starting with '.')"
done
[[ "$owner" =~ ^[A-Za-z0-9][A-Za-z0-9-]*$ ]] || fail "invalid owner '$owner'"
for repo in "${repos[@]}"; do
  [ "$repo" != "$results" ] || fail "$results is the results repository: it cannot be watched as well"
done

if [ -n "$local_dir" ]; then
  [ "$apply_github" -eq 0 ] || fail "--local never touches GitHub: drop --apply-github"
  [ -d "$local_dir" ] || fail "no directory $local_dir"
  local_dir="$(cd "$local_dir" && pwd)"
  sentinel_home="$local_dir/sentinel-home"
  sentinel_exists=1
else
  if [ "$dry_run" -eq 0 ] && [ "$apply_github" -eq 0 ]; then
    fail "refusing: onboarding changes GitHub -- a deploy key with write access on $owner/$results. Pass
      --apply-github, and only after the owner's go in the chat (deploy/PROVISION.md); --dry-run prints the plan"
  fi
  command -v gh >/dev/null || fail "gh is not installed"
  gh auth status >/dev/null 2>&1 || fail "gh is not logged in: gh auth login"
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
readonly key="$ssh_dir/sentinel_$results"
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
  if sentinel_has "$key.pub" && sentinel_has "$key"; then
    ok "key $key: $(as_sentinel ssh-keygen -lf "$key.pub")"
  elif sentinel_has "$key"; then
    fail "$key exists without $key.pub: restore it with ssh-keygen -y, or remove both and re-run"
  else
    step "create the key $key (ed25519, no passphrase)" \
      as_sentinel ssh-keygen -q -t ed25519 -N "" -C "sentinel4rust $results" -f "$key"
    [ "$dry_run" -eq 1 ] || ok "key $key: $(as_sentinel ssh-keygen -lf "$key.pub")"
  fi
}

ensure_host_alias() {
  if sentinel_has "$ssh_config" && as_sentinel grep -qxF "Host github-$results" "$ssh_config"; then
    ok "host alias github-$results in $ssh_config"
    return
  fi
  cat > "$scratch/alias" << EOF

Host github-$results
    HostName github.com
    User git
    IdentityFile $key
    IdentitiesOnly yes
    StrictHostKeyChecking yes
EOF
  step "add the host alias github-$results to $ssh_config" append_as_sentinel "$scratch/alias" "$ssh_config"
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
  [ "$repo" != "$results" ] || return 0
  git --git-dir="$bare" cat-file -e main:.cargo/mutants.toml 2>/dev/null \
    || note "$repo has no .cargo/mutants.toml on main: the sentinel skips it until it has one"
}

# The results repository: it must exist, gh must be its admin, and what it holds is public when it is.
check_results_repo() {
  local info visibility permission
  info="$(gh repo view "$owner/$results" --json visibility,viewerPermission \
    --jq '.visibility + " " + .viewerPermission')" \
    || fail "cannot read $owner/$results on GitHub: create it (gh repo create $owner/$results --public), then re-run"
  visibility="${info% *}"
  permission="${info#* }"
  [ "$permission" = "ADMIN" ] || fail "$owner/$results: a deploy key needs admin, gh has $permission"
  ok "$owner/$results is $visibility, and gh is its admin"
  [ "$visibility" != "PUBLIC" ] \
    || note "every watched repository's survivors and log tails will be public there (deploy/SETUP.md section 7)"
}

# A watched repository: public, so it is read with no credential; no deploy key of a previous design left on it.
check_watched_repo() {
  local repo=$1 visibility titles
  visibility="$(gh repo view "$owner/$repo" --json visibility --jq '.visibility')" \
    || fail "cannot read $owner/$repo on GitHub"
  if [ "$visibility" != "PUBLIC" ]; then
    say "BLOCKED $owner/$repo is $visibility: the sentinel reads watched repositories over https with no credential,"
    say "        so only a public one can be watched. Nothing was changed for it. Ask the owner: make it public, or"
    say "        leave it off the roster (docs/ADRs/ADR-SharedResultsRepository.md)"
    [ "$dry_run" -eq 1 ] || fail "$owner/$repo cannot be onboarded: stop, and ask the owner"
    blocked=1
    return
  fi
  ok "$owner/$repo is PUBLIC: read over https, with no credential"
  if titles="$(gh repo deploy-key list -R "$owner/$repo" --json title --jq '.[].title' 2>/dev/null)" \
    && printf '%s\n' "$titles" | grep -qxF "$key_title"; then
    fail "$owner/$repo has a deploy key titled '$key_title', which the sentinel no longer needs: perhaps an earlier
      onboarding's. Delete it there (Settings, Deploy keys) and re-run; the owner decides"
  fi
  gh api "repos/$owner/$repo/contents/.cargo/mutants.toml?ref=main" --silent 2>/dev/null \
    || note "$repo has no .cargo/mutants.toml on main: the sentinel skips it until it has one"
}

ensure_deploy_key() {
  local repo=$results listing ours="" title key_text read_only
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

# Reading every watched repository with no credential, and writing the results repository with the key.
prove_access() {
  if [ "$dry_run" -eq 1 ]; then
    say "would prove the access as sentinel: $verify_access $results_url ${watched_urls[*]}"
    return
  fi
  say "do    prove the access as sentinel: $verify_access $results_url ${watched_urls[*]}"
  if ! as_sentinel "$verify_access" "$results_url" "${watched_urls[@]}" | sed 's/^/      /'; then
    fail "the access proof failed: nothing was added to the roster"
  fi
}

# The roster's top-level value of $1, or nothing.
roster_value() {
  as_sentinel cat "$roster" | awk -v key="$1" '
    /^[ \t]*\[/ { exit }
    { line = $0; sub(/#.*/, "", line) }
    line ~ "^[ \t]*" key "[ \t]*=" {
      value = line; sub(/^[^=]*=[ \t]*"/, "", value); sub(/".*$/, "", value); print value; exit
    }'
}

# results_url before the roster's first [[repo]]: a key after one would belong to that repository.
ensure_results_url() {
  local current=""
  if sentinel_has "$roster"; then
    current="$(roster_value results_url)"
  fi
  if [ "$current" = "$results_url" ]; then
    ok "results_url = \"$results_url\" in the roster"
    return
  fi
  [ -z "$current" ] || fail "the roster names results_url \"$current\", not \"$results_url\": the owner decides which"
  if [ "$sentinel_exists" -eq 0 ]; then
    say "would set results_url = \"$results_url\" in $roster, before its first [[repo]]"
    return
  fi
  as_sentinel cat "$roster" | awk -v line="results_url = \"$results_url\"" '
    !done && /^[ \t]*\[\[repo\]\]/ { print line; print ""; done = 1 }
    { print }
    END { if (!done) print line }' > "$scratch/roster.new"
  step "set results_url = \"$results_url\" in $roster, before its first [[repo]]" \
    write_as_sentinel "$scratch/roster.new" "$roster"
}

# write_as_sentinel <source> <file>: replaces a file the sentinel owns with one the owner wrote.
write_as_sentinel() {
  as_sentinel tee "$2" < "$1" > /dev/null
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
    proof="# read verified: $today (deploy/verify-access.sh, by deploy/onboard.sh)"
  fi
  printf '\n[[repo]]\nname = "%s"\nurl = "%s"\n%s\n' "$repo" "$url" "$proof" > "$scratch/entry"
  step "append [[repo]] $repo ($url) to $roster" append_as_sentinel "$scratch/entry" "$roster"
  appended=$((appended + 1))
}

watched_url() {
  if [ -n "$local_dir" ]; then
    printf 'file://%s/%s.git' "$local_dir" "$1"
  else
    printf 'https://github.com/%s/%s.git' "$owner" "$1"
  fi
}

appended=0
blocked=0
blocked_repos=()
watched=()
watched_urls=()
if [ -n "$local_dir" ]; then
  results_url="file://$local_dir/$results.git"
else
  results_url="git@github-$results:$owner/$results.git"
fi

say "== the sentinel's side, under $sentinel_home"
ensure_home

say "== the results repository, $results"
if [ -n "$local_dir" ]; then
  check_local_repo "$results"
else
  check_results_repo
fi
ensure_key
ensure_host_alias

for repo in "${repos[@]}"; do
  say "== $repo"
  blocked=0
  if [ -n "$local_dir" ]; then
    check_local_repo "$repo"
  else
    check_watched_repo "$repo"
  fi
  if [ "$blocked" -eq 1 ]; then
    blocked_repos+=("$repo")
    continue
  fi
  watched+=("$repo")
  watched_urls+=("$(watched_url "$repo")")
done

say "== GitHub and the access proof"
if [ -z "$local_dir" ]; then
  ensure_deploy_key
  prove_access
fi

say "== the roster"
ensure_results_url
for i in "${!watched[@]}"; do
  ensure_entry "${watched[$i]}" "${watched_urls[$i]}"
done

say "== done"
if [ "$dry_run" -eq 1 ]; then
  say "dry run: nothing was changed"
fi
if [ "${#blocked_repos[@]}" -gt 0 ]; then
  say "BLOCKED ${blocked_repos[*]}: cannot be onboarded as they are (see above)"
  failed=1
  exit 1
fi
if [ "$dry_run" -eq 0 ] && [ "$appended" -gt 0 ] && [ -z "$local_dir" ] && systemctl is-active --quiet sentinel4rust; then
  note "the service reads the roster on start. Once no run is in progress -- systemctl is-active"
  note "sentinel4rust-mutants@run.service says inactive, or journalctl -u sentinel4rust shows a visit just"
  note "ended -- restart it: sudo systemctl restart sentinel4rust"
fi
