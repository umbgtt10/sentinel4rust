// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust_cli::system::work_dir::WorkDir;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

fn repo() -> OnboardedRepo {
    OnboardedRepo {
        name: String::from("twin4rust"),
        url: String::from("u"),
        branch: String::from("main"),
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
    }
}

fn work_dir() -> WorkDir {
    WorkDir::new(PathBuf::from("/srv/s"))
}

#[test]
fn bare_clone_of_a_repo_is_under_repos_with_a_git_suffix() {
    // Arrange & Act
    let path = work_dir().bare_clone(&repo());

    // Assert
    assert_eq!(path, Path::new("/srv/s/repos/twin4rust.git"));
}

#[test]
fn checkout_of_a_repo_is_under_runs() {
    // Arrange & Act
    let path = work_dir().checkout(&repo());

    // Assert
    assert_eq!(path, Path::new("/srv/s/runs/twin4rust"));
}

#[test]
fn jobs_holds_the_jobs_handed_to_the_run_unit() {
    // Arrange & Act
    let path = work_dir().jobs();

    // Assert
    assert_eq!(path, Path::new("/srv/s/jobs"));
}

#[test]
fn mutants_out_of_a_repo_is_under_out() {
    // Arrange & Act
    let path = work_dir().mutants_out(&repo());

    // Assert
    assert_eq!(path, Path::new("/srv/s/out/twin4rust"));
}

#[test]
fn root_is_the_directory_given() {
    // Arrange & Act
    let work_dir = work_dir();

    // Assert
    assert_eq!(work_dir.root(), Path::new("/srv/s"));
}

#[test]
fn runs_holds_every_checkout() {
    // Arrange & Act
    let path = work_dir().runs();

    // Assert
    assert_eq!(path, Path::new("/srv/s/runs"));
}

#[test]
fn stage_of_a_repo_is_under_stage() {
    // Arrange & Act
    let path = work_dir().stage(&repo());

    // Assert
    assert_eq!(path, Path::new("/srv/s/stage/twin4rust"));
}

#[test]
fn stages_holds_every_staging_area() {
    // Arrange & Act
    let path = work_dir().stages();

    // Assert
    assert_eq!(path, Path::new("/srv/s/stage"));
}

#[test]
fn tmp_of_a_repo_is_under_tmp() {
    // Arrange & Act
    let path = work_dir().tmp(&repo());

    // Assert
    assert_eq!(path, Path::new("/srv/s/tmp/twin4rust"));
}

#[test]
fn tmp_root_holds_every_temporary_directory() {
    // Arrange & Act
    let path = work_dir().tmp_root();

    // Assert
    assert_eq!(path, Path::new("/srv/s/tmp"));
}
