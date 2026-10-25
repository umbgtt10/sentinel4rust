// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::files::file_store::FileStore;
use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;
use std::path::Path;

const BINARY: &str = "cargo-stern4rust";
const MARKER: &str = "REV";
const PIN_REV: &str = "8b0e8e5";
const PIN_URL: &str = "https://github.com/umbgtt10/stern4rust";
const RULE_BROKEN: i32 = 2;

// stern4rust pinned to one pushed commit of its repository, so the gate holds
// the rules of the release in the making rather than whichever version happens
// to be installed. It lives under the repository's own install root, never over
// the global cargo-stern4rust, and is installed again whenever the marker
// beside it does not name the pin.
pub struct SternGate<'a> {
    runner: &'a dyn CommandRunner,
    files: &'a dyn FileStore,
    manifest_path: String,
    packages: Vec<String>,
    install_root: String,
}

impl<'a> SternGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        files: &'a dyn FileStore,
        manifest_path: String,
        packages: Vec<String>,
        install_root: String,
    ) -> Self {
        Self {
            runner,
            files,
            manifest_path,
            packages,
            install_root,
        }
    }

    fn binary(&self) -> String {
        Path::new(&self.install_root)
            .join("bin")
            .join(BINARY)
            .to_string_lossy()
            .into_owned()
    }

    fn install(&self) -> Result<(), GateFailure> {
        let args = vec![
            String::from("install"),
            String::from("--git"),
            String::from(PIN_URL),
            String::from("--rev"),
            String::from(PIN_REV),
            String::from("--locked"),
            String::from("--root"),
            self.install_root.clone(),
            String::from(BINARY),
        ];
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(self.files.write(&self.marker(), &format!("{PIN_REV}\n"))?),
            exit_code => Err(GateFailure::InstallFailed {
                command: format!("cargo {}", args.join(" ")),
                exit_code,
            }),
        }
    }

    fn is_pinned(&self) -> bool {
        self.files.is_file(&self.binary())
            && self
                .files
                .read(&self.marker())
                .is_some_and(|rev| rev.trim() == PIN_REV)
    }

    fn marker(&self) -> String {
        Path::new(&self.install_root)
            .join(MARKER)
            .to_string_lossy()
            .into_owned()
    }
}

impl Gate for SternGate<'_> {
    fn label(&self) -> String {
        String::from("House rules")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.is_pinned() {
            self.install()?;
        }

        let mut args = vec![
            String::from("stern4rust"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }

        // 2 is a rule broken; anything else non-zero is the tool failing to
        // run at all. Kept apart so a bad manifest cannot read as a clean
        // codebase.
        match self.runner.run_streaming(&self.binary(), &args)? {
            Some(0) => Ok(()),
            Some(RULE_BROKEN) => Err(GateFailure::Found(String::from(
                "a house coding rule was broken",
            ))),
            code => Err(GateFailure::CouldNotRun(code)),
        }
    }
}
