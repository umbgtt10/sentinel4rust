// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::repo_names::RepoNames;

#[test]
fn is_branch_accepts_git_branch_names_and_refuses_the_rest() {
    // Arrange
    let good = ["main", "trunk", "release/1.2", "feature_x-y"];
    let bad = ["", "-main", "a..b", "with space", "mutants"];

    // Act & Assert
    for branch in good {
        assert!(RepoNames.is_branch(branch), "{branch}");
    }
    for branch in bad {
        assert!(!RepoNames.is_branch(branch), "{branch}");
    }
}

#[test]
fn is_name_accepts_clone_names_and_refuses_the_rest() {
    // Arrange
    let good = ["twin4rust", "etheram-core", "a_b.c"];
    let bad = ["", ".github", "../x", "a/b", "with space"];

    // Act & Assert
    for name in good {
        assert!(RepoNames.is_name(name), "{name}");
    }
    for name in bad {
        assert!(!RepoNames.is_name(name), "{name}");
    }
}
