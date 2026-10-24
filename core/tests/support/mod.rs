//! Helpers shared by the integration tests: temporary directories and git fixtures.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use sentinel4rust::model::{CommitHash, OnboardedRepo};

/// A directory removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path =
            std::env::temp_dir().join(format!("sentinel4rust-{label}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Runs git, panicking on failure; returns trimmed stdout.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "dev")
        .env("GIT_AUTHOR_EMAIL", "dev@local")
        .env("GIT_COMMITTER_NAME", "dev")
        .env("GIT_COMMITTER_EMAIL", "dev@local")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A bare "remote" plus a working clone used to push commits to it.
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

    pub fn dev(&self) -> PathBuf {
        self.root.path().join("dev")
    }

    pub fn url(&self) -> String {
        format!("file://{}", self.bare().display())
    }

    pub fn repo(&self, name: &str) -> OnboardedRepo {
        OnboardedRepo {
            name: name.to_string(),
            url: self.url(),
            branch: "main".to_string(),
            jobs: 1,
        }
    }

    /// Writes files in the dev clone, commits and pushes `main`; returns the commit.
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

    /// A crate ready to be tested.
    pub fn commit_crate(&self) -> CommitHash {
        self.commit(&[
            ("Cargo.toml", "[package]\nname = \"x\"\n"),
            (".cargo/mutants.toml", "test_workspace = true\n"),
            ("src/lib.rs", "pub fn f() -> bool { true }\n"),
        ])
    }

    /// The remote's ref, or empty.
    pub fn remote_ref(&self, name: &str) -> String {
        Command::new("git")
            .args(["rev-parse", "--verify", "--quiet", name])
            .current_dir(self.bare())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }
}
