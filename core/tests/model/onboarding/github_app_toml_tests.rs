// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::github_app::GithubApp;
use sentinel4rust::model::onboarding::github_app_toml::GithubAppToml;
use sentinel4rust::model::toml::toml_parser::TomlParser;

fn parse(text: &str) -> Result<Option<GithubApp>, String> {
    let doc = TomlParser.parse(text).unwrap();
    GithubAppToml.parse(&doc.root).map_err(|error| error.0)
}

#[test]
fn parse_of_all_three_keys_reads_the_app() {
    // Arrange
    let text = "github_app_id = 7\ngithub_app_installation_id = 8\ngithub_app_key = \"/k\"";

    // Act
    let app = parse(text);

    // Assert
    assert_eq!(
        app,
        Ok(Some(GithubApp {
            app_id: 7,
            installation_id: 8,
            key: String::from("/k"),
        }))
    );
}

#[test]
fn parse_of_no_key_is_no_app() {
    // Arrange & Act
    let app = parse("work_dir = \"/w\"");

    // Assert
    assert_eq!(app, Ok(None));
}

#[test]
fn parse_of_some_keys_or_a_bad_id_fails_naming_the_key() {
    // Arrange
    let cases = [
        ("github_app_key = \"/k\"", "github_app_id"),
        (
            "github_app_id = 7\ngithub_app_installation_id = 0\ngithub_app_key = \"/k\"",
            "github_app_installation_id",
        ),
        (
            "github_app_id = -1\ngithub_app_installation_id = 8\ngithub_app_key = \"/k\"",
            "github_app_id",
        ),
    ];

    // Act & Assert
    for (text, key) in cases {
        let error = parse(text).unwrap_err();
        assert!(error.contains(key), "{key}: {error}");
    }
}
