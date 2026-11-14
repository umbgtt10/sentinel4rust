// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fixed_clock::FixedClock;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::onboarding::github_app::GithubApp;
use sentinel4rust_cli::system::github::app_token::AppToken;
use sentinel4rust_cli::system::github::secret_dir::SecretDir;
use sentinel4rust_cli::system::github::token_source::TokenSource;
use sentinel4rust_cli::system::process::system_command_runner::SystemCommandRunner;
use std::cell::Cell;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

// 2027-01-15T08:00:00Z.
const NOW: i64 = 1_800_000_000;

fn app(key: &Path) -> GithubApp {
    GithubApp {
        app_id: 7,
        installation_id: 8,
        key: key.to_string_lossy().into_owned(),
    }
}

fn base64_url_decode(text: &str) -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let values: Vec<u32> = text
        .bytes()
        .map(|b| u32::try_from(alphabet.iter().position(|a| *a == b).unwrap()).unwrap())
        .collect();
    let mut bytes = Vec::new();
    for chunk in values.chunks(4) {
        let group = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, v)| acc | v << (18 - 6 * i));
        for i in 0..chunk.len() - 1 {
            bytes.push(u8::try_from((group >> (16 - 8 * i)) & 0xff).unwrap());
        }
    }
    bytes
}

fn token_reply(token: &str, expires: &str) -> String {
    format!(r#"{{"token":"{token}","expires_at":"{expires}","permissions":{{"contents":"read"}}}}"#)
}

#[test]
fn jwt_signed_with_openssl_is_verified_by_the_public_key() {
    // Arrange
    let dir = TempDir::new("app-key");
    let key = dir.path().join("app.pem");
    let public = dir.path().join("app.pub");
    let openssl = |args: &[&str]| {
        assert!(
            Command::new("openssl")
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    };
    openssl(&[
        "genpkey",
        "-quiet",
        "-algorithm",
        "RSA",
        "-pkeyopt",
        "rsa_keygen_bits:2048",
        "-out",
        &key.to_string_lossy(),
    ]);
    openssl(&[
        "pkey",
        "-in",
        &key.to_string_lossy(),
        "-pubout",
        "-out",
        &public.to_string_lossy(),
    ]);
    let token = AppToken::new(
        Rc::new(SystemCommandRunner),
        Rc::new(FixedClock(Cell::new(NOW))),
        app(&key),
        SecretDir::new(dir.path().join("github")),
    );

    // Act
    let jwt = token.jwt().unwrap();

    // Assert
    let (input, signature) = jwt.rsplit_once('.').unwrap();
    fs::write(dir.path().join("input"), input).unwrap();
    fs::write(dir.path().join("signature"), base64_url_decode(signature)).unwrap();
    let verified = Command::new("openssl")
        .args([
            "dgst",
            "-sha256",
            "-verify",
            &public.to_string_lossy(),
            "-signature",
        ])
        .arg(dir.path().join("signature"))
        .arg(dir.path().join("input"))
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&verified.stdout).trim(),
        "Verified OK"
    );
    assert!(
        input.starts_with("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9."),
        "{input}"
    );
}

#[test]
fn token_mints_through_curl_and_reuses_it_until_five_minutes_before_expiry() {
    // Arrange
    let dir = TempDir::new("app-token");
    let secrets = SecretDir::new(dir.path().join("github"));
    secrets.write("jwt.sig", &[1, 2, 3]).unwrap();
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(""),
        ok(&token_reply("ghs_one", "2027-01-15T09:00:00Z")),
        ok(""),
        ok(&token_reply("ghs_two", "2027-01-15T10:00:00Z")),
    ]));
    let clock = Rc::new(FixedClock(Cell::new(NOW)));
    let key = dir.path().join("app.pem");
    let token = AppToken::new(runner.clone(), clock.clone(), app(&key), secrets.clone());
    let expiry = NOW + 3600;

    // Act
    let first = token.token().unwrap();
    clock.0.set(expiry - 301);
    let reused = token.token().unwrap();
    clock.0.set(expiry - 300);
    let minted = token.token().unwrap();

    // Assert
    assert_eq!(
        (first.as_str(), reused.as_str(), minted.as_str()),
        ("ghs_one", "ghs_one", "ghs_two")
    );
    let commands = runner.commands();
    assert_eq!(commands.len(), 4);
    assert_eq!(commands[0].program(), "openssl");
    assert_eq!(
        commands[0].args(),
        [
            "dgst",
            "-sha256",
            "-sign",
            key.to_string_lossy().as_ref(),
            "-out",
            secrets.path("jwt.sig").to_string_lossy().as_ref(),
            secrets.path("jwt.input").to_string_lossy().as_ref()
        ]
    );
    assert_eq!(commands[1].program(), "curl");
    let header = format!("@{}", secrets.path("app.header").display());
    assert_eq!(
        commands[1].args(),
        [
            "--silent",
            "--show-error",
            "--fail-with-body",
            "--max-time",
            "60",
            "--request",
            "POST",
            "--header",
            header.as_str(),
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            "https://api.github.com/app/installations/8/access_tokens"
        ]
    );
    let sent = fs::read_to_string(secrets.path("app.header")).unwrap();
    assert!(sent.starts_with("Authorization: Bearer eyJ"), "{sent}");
}

#[test]
fn token_refused_names_curl_and_what_github_answered() {
    // Arrange
    let dir = TempDir::new("app-refused");
    let secrets = SecretDir::new(dir.path().join("github"));
    secrets.write("jwt.sig", &[1]).unwrap();
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(""),
        failed(22, "curl: (22) The requested URL returned error: 401"),
    ]));
    let token = AppToken::new(
        runner,
        Rc::new(FixedClock(Cell::new(NOW))),
        app(&dir.path().join("app.pem")),
        secrets,
    );

    // Act
    let error = token.token().unwrap_err();

    // Assert
    assert!(error.0.starts_with("curl"), "{error}");
    assert!(error.0.contains("401"), "{error}");
}
