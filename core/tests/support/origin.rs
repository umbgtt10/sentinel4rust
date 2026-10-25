// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::git_shell::git;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// A bare "remote" plus a working clone used to push commits to it.
pub struct Origin {
    pub root: TempDir,
}

impl Origin {
    pub fn new() -> Self {
        let root = TempDir::new("origin");
        fs::create_dir_all(root.path().join("remote.git")).unwrap();
        git(
            &root.path().join("remote.git"),
            &["init", "--quiet", "--bare", "-b", "main"],
        );
        git(root.path(), &["clone", "--quiet", "remote.git", "dev"]);
        git(
            &root.path().join("dev"),
            &["checkout", "--quiet", "-b", "main"],
        );
        Self { root }
    }

    pub fn bare(&self) -> PathBuf {
        self.root.path().join("remote.git")
    }

    // Writes files in the dev clone, commits and pushes `main`; returns the commit.
    pub fn commit(&self, files: &[(&str, &str)]) -> CommitHash {
        let dev = self.dev();
        for (path, contents) in files {
            let file = dev.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, contents).unwrap();
        }
        git(&dev, &["add", "-A"]);
        git(
            &dev,
            &["commit", "--quiet", "--allow-empty", "-m", "change"],
        );
        git(&dev, &["push", "--quiet", "origin", "main"]);
        CommitHash::parse(&git(&dev, &["rev-parse", "HEAD"])).unwrap()
    }

    // A crate ready to be tested.
    pub fn commit_crate(&self) -> CommitHash {
        self.commit(&[
            ("Cargo.toml", "[package]\nname = \"x\"\n"),
            (".cargo/mutants.toml", "test_workspace = true\n"),
            ("src/lib.rs", "pub fn f() -> bool { true }\n"),
        ])
    }

    pub fn dev(&self) -> PathBuf {
        self.root.path().join("dev")
    }

    // The remote's ref, or empty.
    pub fn remote_ref(&self, name: &str) -> String {
        Command::new("git")
            .args(["rev-parse", "--verify", "--quiet", name])
            .current_dir(self.bare())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }

    pub fn repo(&self, name: &str) -> OnboardedRepo {
        OnboardedRepo {
            name: name.to_string(),
            url: self.url(),
            branch: "main".to_string(),
            jobs: 1,
        }
    }

    pub fn url(&self) -> String {
        format!("file://{}", self.bare().display())
    }
}
