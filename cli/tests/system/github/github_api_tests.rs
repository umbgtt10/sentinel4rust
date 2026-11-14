// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fixed_token::FixedToken;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust_cli::system::github::github_api::GithubApi;
use sentinel4rust_cli::system::github::secret_dir::SecretDir;
use std::fs;
use std::rc::Rc;

fn api(runner: &Rc<ScriptedRunner>, dir: &TempDir) -> (GithubApi, SecretDir) {
    let secrets = SecretDir::new(dir.path().join("github"));
    (
        GithubApi::new(
            runner.clone(),
            Rc::new(FixedToken(String::from("ghs_t"))),
            secrets.clone(),
        ),
        secrets,
    )
}

#[test]
fn exists_reads_the_status_code() {
    // Arrange
    let dir = TempDir::new("api-exists");
    let runner = Rc::new(ScriptedRunner::new(vec![ok("200"), ok("404"), ok("500")]));
    let (api, secrets) = api(&runner, &dir);
    let url = "https://api.github.com/repos/o/r/contents/.cargo/mutants.toml?ref=main";

    // Act
    let answers = [api.exists(url), api.exists(url), api.exists(url)];

    // Assert
    assert_eq!(answers[0], Ok(true));
    assert_eq!(answers[1], Ok(false));
    assert!(answers[2].as_ref().unwrap_err().0.contains("500"));
    let header = format!("@{}", secrets.path("api.header").display());
    assert_eq!(
        runner.commands()[0].args(),
        [
            "--silent",
            "--show-error",
            "--max-time",
            "60",
            "--output",
            "/dev/null",
            "--write-out",
            "%{http_code}",
            "--header",
            header.as_str(),
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            url
        ]
    );
}

#[test]
fn get_failing_names_curl_and_what_github_answered() {
    // Arrange
    let dir = TempDir::new("api-fail");
    let runner = Rc::new(ScriptedRunner::new(vec![failed(
        22,
        "curl: (22) The requested URL returned error: 403",
    )]));
    let (api, _) = api(&runner, &dir);

    // Act
    let error = api
        .get("https://api.github.com/installation/repositories")
        .unwrap_err();

    // Assert
    assert!(error.0.starts_with("curl"), "{error}");
    assert!(error.0.contains("403"), "{error}");
}

#[test]
fn get_sends_the_token_from_a_private_file_and_returns_the_body() {
    // Arrange
    let dir = TempDir::new("api-get");
    let runner = Rc::new(ScriptedRunner::new(vec![ok("{\"total_count\":0}")]));
    let (api, secrets) = api(&runner, &dir);
    let url = "https://api.github.com/installation/repositories?per_page=100&page=1";

    // Act
    let body = api.get(url).unwrap();

    // Assert
    assert_eq!(body, "{\"total_count\":0}");
    let header = format!("@{}", secrets.path("api.header").display());
    assert_eq!(
        runner.commands()[0].args(),
        [
            "--silent",
            "--show-error",
            "--fail-with-body",
            "--max-time",
            "60",
            "--header",
            header.as_str(),
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            url
        ]
    );
    assert_eq!(
        fs::read_to_string(secrets.path("api.header")).unwrap(),
        "Authorization: Bearer ghs_t\n"
    );
    assert!(
        runner.commands()[0]
            .args()
            .iter()
            .all(|arg| !arg.contains("ghs_t"))
    );
}
