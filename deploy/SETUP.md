# Setting up the machine

The short version of `docs/DESIGN.md` section 5, in order.

## 1. User and tools

```sh
sudo useradd --create-home --shell /bin/bash sentinel
sudo -iu sentinel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
cargo install cargo-mutants --version 27.1.0 --locked
cargo install --path /path/to/sentinel4rust/core --locked      # or: cargo install sentinel4rust
mkdir -p ~/.config/sentinel4rust && cp deploy/roster.example.toml ~/.config/sentinel4rust/roster.toml
ssh-keyscan github.com >> ~/.ssh/known_hosts                     # check against GitHub's published fingerprints
```

## 2. Per repository

On the machine, as `sentinel`:

```sh
ssh-keygen -t ed25519 -N "" -C "sentinel4rust <repo>" -f ~/.ssh/sentinel_<repo>
chmod 700 ~/.ssh && chmod 600 ~/.ssh/sentinel_*
cat >> ~/.ssh/config <<EOF
Host github-<repo>
    HostName github.com
    User git
    IdentityFile ~/.ssh/sentinel_<repo>
    IdentitiesOnly yes
EOF
```

On GitHub:

- Settings → Deploy keys → Add: `~/.ssh/sentinel_<repo>.pub`, **Allow write access**.
- Settings → Rules → Rulesets → New branch ruleset `sentinel: only mutants`: enforcement **Active**; target
  **all branches**, **exclude** `mutants`; **Restrict creations**, **Restrict updates**, **Restrict deletions**,
  **Block force pushes**; bypass list: yourself only, **no deploy keys**.
- The repository needs a `.cargo/mutants.toml` on `main` (it is skipped without one).

Verify, then add it to the roster:

```sh
deploy/verify-access.sh git@github-<repo>:umbgtt10/<repo>.git
```

## 3. The service

```sh
sudo cp deploy/sentinel4rust.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo -iu sentinel sentinel4rust --roster ~/.config/sentinel4rust/roster.toml --once   # first pass by hand
sudo systemctl enable --now sentinel4rust
journalctl -u sentinel4rust -f
```

The `--once` pass and the service share a lock: stop the service before a manual pass.
