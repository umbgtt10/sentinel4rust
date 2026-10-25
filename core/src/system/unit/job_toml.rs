// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A command, written for the run unit and read back inside it.

use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

use crate::model::command::Command;
use crate::model::toml::toml_parser::TomlParser;
use crate::model::toml::toml_writer::TomlWriter;
use crate::system::unit::job_error::JobError;

const ROOT_KEYS: &[&str] = &["program", "dir", "timeout_seconds", "capture_bytes"];

/// `job.toml`: a command's program, directory, deadline, how much of its output to keep, arguments (`[[arg]]`)
/// and extra environment (`[[env]]`).
#[derive(Debug, Clone, Copy, Default)]
pub struct JobToml;

impl JobToml {
    /// The command `text` describes.
    pub fn parse(&self, text: &str) -> Result<Command, JobError> {
        let doc = TomlParser.parse(text)?;
        doc.root.only(ROOT_KEYS)?;
        if let Some(name) = doc
            .arrays
            .keys()
            .find(|name| *name != "arg" && *name != "env")
        {
            return Err(JobError(format!("unknown section `[[{name}]]`")));
        }
        let program = doc.root.required_string("program")?;
        let dir = doc.root.required_string("dir")?;
        let timeout = u64::try_from(doc.root.required_integer("timeout_seconds")?)
            .map_err(|_| JobError("negative `timeout_seconds`".to_string()))?;
        let args = doc
            .arrays
            .get("arg")
            .into_iter()
            .flatten()
            .map(|table| table.required_string("value"))
            .collect::<Result<Vec<String>, _>>()?;
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let mut command = Command::new(
            &program,
            &arg_refs,
            Path::new(&dir),
            Duration::from_secs(timeout),
        );
        for table in doc.arrays.get("env").into_iter().flatten() {
            command = command.with_env(
                &table.required_string("name")?,
                &table.required_string("value")?,
            );
        }
        if let Some(bytes) = doc.root.integer("capture_bytes")? {
            let bytes = usize::try_from(bytes)
                .map_err(|_| JobError("negative `capture_bytes`".to_string()))?;
            command = command.with_capture_limit(bytes);
        }
        Ok(command)
    }

    /// The `job.toml` text for `command`.
    pub fn render(&self, command: &Command) -> String {
        let mut out = format!(
            "program = {}\ndir = {}\ntimeout_seconds = {}\n",
            TomlWriter.quote(command.program()),
            TomlWriter.quote(&command.dir().to_string_lossy()),
            command.timeout().as_secs()
        );
        if let Some(bytes) = command.capture_limit() {
            let _ = writeln!(out, "capture_bytes = {bytes}");
        }
        for arg in command.args() {
            let _ = write!(out, "\n[[arg]]\nvalue = {}\n", TomlWriter.quote(arg));
        }
        for (name, value) in command.env() {
            let _ = write!(
                out,
                "\n[[env]]\nname = {}\nvalue = {}\n",
                TomlWriter.quote(name),
                TomlWriter.quote(value)
            );
        }
        out
    }
}
