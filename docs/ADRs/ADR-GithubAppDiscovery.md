# ADR — A GitHub App discovers and reads the repositories

## Status

Accepted.

## Context

Watching a repository took an onboarding of its own: a key and a host alias for a private one, a deploy key added on
GitHub after the owner's go, a roster entry, and a restart of the service to read it. Every new repository repeated
it. On 2026-11-14 the owner asked for the onboarding to go: the sentinel should find the repositories itself, and test
those that ask to be tested. Deploy keys cannot do that -- each is bound to one repository by design -- so finding
them, and reading the private ones among them, needs one credential that sees the whole account.

## Decision

A GitHub App, installed on all the owner's repositories with **read-only** access to their contents, is the
sentinel's way to find and read them: the roster names it (`github_app_id`, `github_app_installation_id`,
`github_app_key`), and the sentinel watches every repository the installation lists that is not archived and has a
`.cargo/mutants.toml` on its default branch, refreshed every `discovery_minutes`, over https with an installation
token it mints itself. Writing results stays as it was: the one write deploy key on the results repository.

- **The token.** `AppToken` (`cli/src/system/github/`) builds the App's JWT -- RS256, issued a minute back, valid
  nine minutes ahead, `iss` the App id; the header and claims come from the core's `AppJwt` -- has
  `openssl dgst -sha256 -sign <key>` sign it, and exchanges it with `curl` for an installation token
  (`POST /app/installations/<id>/access_tokens`), kept until five minutes before it expires. Every file it writes --
  the signing input, the signature, the request headers -- is mode 600 in `sentinel4rust-github/`, mode 700, beside
  the App's key, under `~sentinel/.ssh`, which the run unit cannot read. No token is ever on a command line: curl
  reads its `Authorization` header from a file (`-H @file`).
- **Discovery.** `GithubAppDiscovery` pages through `/installation/repositories` (100 a page) and asks the contents
  API, per repository not archived, whether `.cargo/mutants.toml` is on its default branch (200 or 404). The core's
  `WatchList` puts the roster's own `[[repo]]` entries first -- an explicit entry wins over a discovered one of the
  same name, which is how a repository gets its own `jobs`, `run_timeout_hours`, branch or URL -- then the
  discovered ones by name, never the results repository, nor a name that cannot name a clone. `DiscoveryRefresh`
  runs discovery at start and every `discovery_minutes` (60 by default), keeps the last list when it fails and tries
  again after `pause_seconds`, and the loop sleeps no later than the next discovery. Adding a repository is adding a
  `.cargo/mutants.toml` to it: no roster change, no restart.
- **git.** `GitTokenRunner`, a `CommandRunner` around the real one, gives git's network commands (`clone`, `fetch`,
  `ls-remote`, `push`) the token through the environment -- `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_0`,
  `GIT_CONFIG_VALUE_0`: `http.https://github.com/.extraheader` = `AUTHORIZATION: basic` and base64 of
  `x-access-token:<token>` -- which only the sentinel's own processes can read. It runs every other command as it is,
  and a command it has no token for unchanged. cargo-mutants' runner is never wrapped.
- **Private git dependencies.** The run unit's unit file points git at a configuration of its own,
  `GIT_CONFIG_GLOBAL=~sentinel/.config/sentinel4rust/run-unit.gitconfig`, which `deploy/provision.sh` writes with one
  rule: `https://github.com/<owner>/` is fetched from `file://<work_dir>/repos/` (git finds `<name>.git` from
  `<name>`), and `~sentinel/.cargo/config.toml` sets `net.git-fetch-with-cli`, so cargo fetches with git, which
  follows it. A build in the run unit resolves a git dependency on any watched repository of the owner's through the
  sentinel's clone of it, holding no key; the clone keeps the branch at `refs/heads/<branch>` for this. The rule
  applies in the run unit only, so the service's own https fetches go to GitHub.

## Forcing constraints / Evidence

Deploy keys are per repository by design, and GitHub will not accept one on two repositories: discovering, and
reading new private repositories without an onboarding, cannot be built on them. An App's installation token covers
every repository it is installed on, including those created later, and is minted from the App's private key
whenever needed, so nothing expires by hand.

The local-clone redirect was proven before it was generalised: on 2026-11-14 etheram-ibft's build in the run unit
resolved the private etheram-core through the sentinel's clone, with a per-repository redirect in
`~sentinel/.gitconfig` (published 152d0de: 333 missed, 2388 caught), and a scratch crate showed cargo with
`net.git-fetch-with-cli` following the redirect while `Cargo.lock` kept the original URL. One prefix rule resolves
both `https://github.com/<owner>/<name>` and `.../<name>.git` to the bare clone `<name>.git`, checked the same day.

## Rejected alternatives

- **A fine-grained personal access token.** Equally read-only and account-wide, but it expires and is renewed by
  hand: a sentinel that goes quiet because a token lapsed is the failure it exists to prevent.
- **A crypto crate for RS256.** `openssl` is on every machine the sentinel runs on, and run through the
  `CommandRunner` like git and cargo it is tested with scripted commands, and once against the real tool with a key
  generated for the test (`ADR-NoDependencies`).
- **An App with write access, publishing too.** An App's permissions apply to every repository it is installed on:
  write to the results repository would be write to all of them. The results repository keeps its own write deploy
  key (`ADR-SharedResultsRepository`).
- **A redirect per repository in `~sentinel/.gitconfig`.** It worked while watched private repositories were fetched
  through SSH aliases. With every watched URL `https://github.com/...`, a redirect there would turn the service's own
  fetch of a repository onto its own clone; the rule belongs to the run unit alone.

## Consequences

One credential on the machine reads every repository of the account: read-only, minted hourly, its key and tokens
kept under `~sentinel/.ssh`, where the run unit cannot read them. The opt-in is unchanged -- a repository without
`.cargo/mutants.toml` on its default branch is not watched -- and so is the record: a repository's folder in the
results repository keeps its name, so moving a machine from per-repository onboarding to the App reruns nothing.
Discovery costs one API call per page and one per repository not archived, every `discovery_minutes`, well within an
installation's rate limit. `serde_json` reads the API's answers in `cli` (`ADR-NoDependencies`). In the run unit, a
git dependency on a repository of the owner's that is not watched -- and so has no clone -- now fails
(`docs/OPEN_POINTS.md` point 22), and a dependant visited before its dependency's first clone backs off and succeeds
on a later visit. Without an App the roster's `[[repo]]` entries are all there is, and per-repository onboarding
(`deploy/onboard.sh <repo>...`) still serves.

## Enforcement

`AppJwt` and `Base64` (`core/tests/model/github/`) pin the JWT's header, claims and encodings; `GithubAppToml` and
`TomlRoster` the roster keys; `WatchList` and `DiscoveryRefresh` (`core/tests/patrol/`) the merge, the refresh, its
journal lines and the retry; the sentinel's tests that the loop visits what discovery found and sleeps no later than
the next discovery. In `cli/tests/system/github/`, `AppToken` is tested against the real `openssl` -- its JWT verifies
with the public half of a key made for the test -- and against scripted curl for the exchange and the cache;
`GithubApi` that the token reaches curl through a mode-600 file and never its arguments; `GitTokenRunner` that only
git's network commands carry it; `GithubAppDiscovery` the paging and the filters. `--status` discovers too and exits 1
when discovery fails.

## Related

- `ADR-SharedResultsRepository.md` -- where the results go, with the one write key.
- `ADR-FiltersLiveInEachRepository.md` -- `.cargo/mutants.toml`, now also the opt-in to being discovered.
- `ADR-NoDependencies.md` -- `serde_json` in `cli`.
- `deploy/SETUP.md` section 2a, `deploy/PROVISION.md` -- creating the App, and moving a machine to it.
