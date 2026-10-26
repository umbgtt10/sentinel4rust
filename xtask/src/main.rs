// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::args;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use xtask::crap::crap_report_parser::CrapReportParser;
use xtask::files::system_file_store::SystemFileStore;
use xtask::gates::boundary::boundary_rule::BoundaryRule;
use xtask::gates::boundary::crate_root_attributes::CrateRootAttributes;
use xtask::gates::boundary::disk_source_files::DiskSourceFiles;
use xtask::gates::boundary::package_dependencies::PackageDependencies;
use xtask::gates::boundary_gate::BoundaryGate;
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

const CLI_PACKAGE: &str = "sentinel4rust";
const CORE_PACKAGE: &str = "sentinel4rust-core";
const XTASK_PACKAGE: &str = "xtask";
const ALLOWED_CRATES: [&str; 0] = [];
const CORE_DIRS: [[&str; 2]; 2] = [["core", "src"], ["core", "tests"]];
const CORE_ROOT: [&str; 3] = ["core", "src", "lib.rs"];
const DRY_EXCLUDES: [&[&str]; 3] = [&["xtask"], &["core", "tests"], &["cli", "tests"]];
const FORBIDDEN_MODULES: [&str; 7] = [
    "std::env",
    "std::fs",
    "std::io",
    "std::net",
    "std::os",
    "std::process",
    "std::thread",
];
const FORBIDDEN_MACROS: [&str; 5] = ["dbg", "eprint", "eprintln", "print", "println"];
const REQUIRED_ATTRIBUTES: [&str; 2] = ["no_std", "forbid(unsafe_code)"];
const FORBIDDEN_METHODS: [&str; 10] = [
    "canonicalize",
    "exists",
    "is_dir",
    "is_file",
    "is_symlink",
    "metadata",
    "read_dir",
    "read_link",
    "symlink_metadata",
    "try_exists",
];
const CRAP_THRESHOLD: &str = "15";
const ICEBERG_THRESHOLD: &str = "7";
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

    // Every member, so the crate that enforces the bar is held to it too.
    let packages = vec![
        String::from(CORE_PACKAGE),
        String::from(CLI_PACKAGE),
        String::from(XTASK_PACKAGE),
    ];

    // First: core stays free of technology -- no dependency in its manifest,
    // `no_std` and `forbid(unsafe_code)` on its root, and no file system,
    // process, thread, environment or stdout in its tests -- so mutation testing
    // it needs nothing but its own tests. A breach is fixed by moving code into
    // cli, which changes what every later gate, stern4rust's included, measures;
    // and it costs a second.
    let root = workspace_root();
    let sources = DiskSourceFiles::new();
    let dependencies = PackageDependencies::new();
    let attributes = CrateRootAttributes::new();
    let boundary = BoundaryGate::new(
        &runner,
        &sources,
        &dependencies,
        &attributes,
        manifest_path.clone(),
        BoundaryRule {
            package: String::from(CORE_PACKAGE),
            source_dirs: CORE_DIRS.iter().map(|dir| below(&root, dir)).collect(),
            allowed_crates: ALLOWED_CRATES.map(String::from).to_vec(),
            crate_root: below(&root, &CORE_ROOT),
            required_attributes: REQUIRED_ATTRIBUTES.map(String::from).to_vec(),
            forbidden_modules: FORBIDDEN_MODULES.map(String::from).to_vec(),
            forbidden_macros: FORBIDDEN_MACROS.map(String::from).to_vec(),
            forbidden_methods: FORBIDDEN_METHODS.map(String::from).to_vec(),
        },
    );

    // Second: its corrections are renames, file moves and directory splits,
    // so a layout it is about to reject is a layout the rest would measure
    // for nothing. Every rule holds, so there is no baseline; it runs the pinned
    // stern4rust it installs under target/, never the global one.
    let stern = SternGate::new(
        &runner,
        &files,
        manifest_path.clone(),
        packages.clone(),
        root.join(STERN_INSTALL_ROOT).to_string_lossy().into_owned(),
    );

    // Third: every declared dependency used, in the section that uses it. Its
    // corrections edit manifests only, so they move nothing the gates behind it
    // measure, and it costs a second.
    let shear = ShearGate::new(&runner, manifest_path.clone());

    // Fourth, for the same reason stern4rust runs early: removing a duplicate
    // moves code between files. One run over both crates' sources, so a copy
    // from one into the other is seen -- tests repeat their arrangement by
    // design, and xtask is not the service -- with zero ceilings against a
    // baseline of what was already duplicated when the gate arrived.
    let dry = DryGate::new(
        &runner,
        root.to_string_lossy().into_owned(),
        DRY_EXCLUDES
            .iter()
            .map(|exclude| below(&root, exclude).to_string_lossy().into_owned())
            .collect(),
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

    let gates: Vec<&dyn Gate> = vec![
        &boundary,
        &stern,
        &shear,
        &dry,
        &crap,
        &twin,
        &iceberg,
        &end_to_end,
    ];

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

// Joined a component at a time, so the path carries the platform's separator
// throughout -- dry4rust matches its exclusions as text.
fn below(root: &Path, components: &[&str]) -> PathBuf {
    components
        .iter()
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one directory below the workspace root")
        .to_path_buf()
}
