// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::ScriptedRunner;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust_cli::system::github::curl::Curl;
use std::path::Path;
use std::rc::Rc;

#[test]
fn get_refused_with_a_body_carries_githubs_answer() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![CommandResult::Exited {
        code: Some(22),
        stdout: String::from("{\"message\":\"Bad credentials\"}\n"),
        stderr: String::from("curl: (22) The requested URL returned error: 401"),
    }]));
    let curl = Curl::new(runner);

    // Act
    let error = curl
        .get(Path::new("/s/api.header"), "https://api.github.com/x")
        .unwrap_err();

    // Assert
    assert!(
        error.0.starts_with("curl https://api.github.com/x: "),
        "{error}"
    );
    assert!(
        error.0.ends_with("{\"message\":\"Bad credentials\"}"),
        "{error}"
    );
}

#[test]
fn post_runs_in_the_header_files_directory() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![CommandResult::Exited {
        code: Some(0),
        stdout: String::from("{}"),
        stderr: String::new(),
    }]));
    let curl = Curl::new(runner.clone());

    // Act
    let body = curl.post(Path::new("/s/app.header"), "https://api.github.com/y");

    // Assert
    assert_eq!(body, Ok(String::from("{}")));
    assert_eq!(runner.commands()[0].program(), "curl");
    assert_eq!(runner.commands()[0].dir(), "/s");
}

#[test]
fn status_returns_the_code_curl_wrote() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![CommandResult::Exited {
        code: Some(0),
        stdout: String::from("404"),
        stderr: String::new(),
    }]));
    let curl = Curl::new(runner);

    // Act
    let status = curl.status(Path::new("/s/api.header"), "https://api.github.com/z");

    // Assert
    assert_eq!(status, Ok(String::from("404")));
}
