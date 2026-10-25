// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::args;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use xtask::crap::crap_report_parser::CrapReportParser;
use xtask::files::system_file_store::SystemFileStore;
use xtask::gates::crap_gate::CrapGate;
use xtask::gates::dry_gate::DryGate;
use xtask::gates::end_to_end_gate::EndToEndGate;
use xtask::gates::gate::Gate;
use xtask::gates::iceberg_gate::IcebergGate;
use xtask::gates::shear_gate::ShearGate;
use xtask::gates::stage2::Stage2;
use xtask::gates::stern_gate::SternGate;
use xtask::gates::twin_gate::TwinGate;
use xtask::process::system_command_runner::SystemCommandRunner;

const CORE_PACKAGE: &str = "sentinel4rust";
const XTASK_PACKAGE: &str = "xtask";
const CRAP_THRESHOLD: &str = "15";
const ICEBERG_THRESHOLD: &str = "7";
const DRY_PATH: &str = "core/src";
const DRY_BASELINE: &str = "dry4rust-baseline.json";
const DRY_MIN_NODES: &str = "25";
const CARGO_MUTANTS_VERSION: &str = "27.1.0";
const STERN_INSTALL_ROOT: &str = "target/stern4rust";

// Reading the real process argv and wiring the concrete runner are the two
// things no test can reach, so they are all this binary does.
fn main() -> ExitCode {
    if args().nth(1).as_deref() == Some("stage2") {
        run_stage2()
    } else {
        eprintln!("usage: cargo xtask stage2");
        ExitCode::FAILURE
    }
}

fn run_stage2() -> ExitCode {
    let manifest_path = workspace_manifest_path();
    let runner = SystemCommandRunner::new();
    let files = SystemFileStore::new();
    let parser = CrapReportParser::new();

    // Both members, so the crate that enforces the bar is held to it too.
    let packages = vec![String::from(CORE_PACKAGE), String::from(XTASK_PACKAGE)];

    // First: its corrections are renames, file moves and directory splits,
    // so a layout it is about to reject is a layout the rest would measure
    // for nothing. Every rule holds, so there is no baseline; it runs the pinned
    // stern4rust it installs under target/, never the global one.
    let root = workspace_root();
    let stern = SternGate::new(
        &runner,
        &files,
        manifest_path.clone(),
        packages.clone(),
        root.join(STERN_INSTALL_ROOT).to_string_lossy().into_owned(),
    );

    // Second: every declared dependency used, in the section that uses it. Its
    // corrections edit manifests only, so they move nothing the gates behind it
    // measure, and it costs a second.
    let shear = ShearGate::new(&runner, manifest_path.clone());

    // Third, for the same reason stern4rust is first: removing a duplicate
    // moves code between files. The core's source only, with zero ceilings
    // against a baseline of what was already duplicated when the gate arrived.
    let dry = DryGate::new(
        &runner,
        root.join(DRY_PATH).to_string_lossy().into_owned(),
        root.join(DRY_BASELINE).to_string_lossy().into_owned(),
        String::from(DRY_MIN_NODES),
    );

    let crap = CrapGate::new(
        &runner,
        &parser,
        manifest_path.clone(),
        packages.clone(),
        String::from(CRAP_THRESHOLD),
    );

    // Every source file has its mirrored test file.
    let twin = TwinGate::new(&runner, manifest_path.clone(), packages.clone());

    // A ratchet just above the worst file (6.78 when the restructure ended),
    // lowered as it improves and never raised.
    let iceberg = IcebergGate::new(
        &runner,
        manifest_path.clone(),
        packages,
        String::from(ICEBERG_THRESHOLD),
    );

    // Last: the slowest, and a behaviour check rather than a measurement.
    let end_to_end = EndToEndGate::new(&runner, manifest_path, String::from(CARGO_MUTANTS_VERSION));

    let gates: Vec<&dyn Gate> = vec![&stern, &shear, &dry, &crap, &twin, &iceberg, &end_to_end];

    match Stage2::new(gates).run() {
        Ok(()) => {
            println!("\nsentinel4rust Stage 2 passed!");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("\nFailed: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_manifest_path() -> String {
    workspace_root()
        .join("Cargo.toml")
        .to_string_lossy()
        .into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one directory below the workspace root")
        .to_path_buf()
}
