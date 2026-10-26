// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! How a job's command ended, written inside the run unit and read back outside it.

use crate::model::command_result::CommandResult;
use crate::model::toml::toml_parser::TomlParser;
use crate::model::toml::toml_writer::TomlWriter;
use crate::protocol::unit::job_error::JobError;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

/// `result.toml`: `kind` -- `exited` with its `code`, `signal`, `timed-out`, or `failed-to-start` with its `message`;
/// the output lies beside it, in `stdout` and `stderr`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResultToml;

impl ResultToml {
    /// The result `text` describes, with the output read beside it.
    pub fn parse(&self, text: &str, stdout: &str, stderr: &str) -> Result<CommandResult, JobError> {
        let doc = TomlParser.parse(text)?;
        let output = || (stdout.to_string(), stderr.to_string());
        match doc.root.required_string("kind")?.as_str() {
            "exited" => {
                let code = i32::try_from(doc.root.required_integer("code")?)
                    .map_err(|_| JobError("`code` out of range".to_string()))?;
                let (stdout, stderr) = output();
                Ok(CommandResult::Exited {
                    code: Some(code),
                    stdout,
                    stderr,
                })
            }
            "signal" => {
                let (stdout, stderr) = output();
                Ok(CommandResult::Exited {
                    code: None,
                    stdout,
                    stderr,
                })
            }
            "timed-out" => {
                let (stdout, stderr) = output();
                Ok(CommandResult::TimedOut { stdout, stderr })
            }
            "failed-to-start" => Ok(CommandResult::FailedToStart(
                doc.root.required_string("message")?,
            )),
            other => Err(JobError(format!("unknown result kind `{other}`"))),
        }
    }

    /// The `result.toml` text for `result`; its output is written apart.
    pub fn render(&self, result: &CommandResult) -> String {
        match result {
            CommandResult::Exited {
                code: Some(code), ..
            } => format!("kind = \"exited\"\ncode = {code}\n"),
            CommandResult::Exited { code: None, .. } => "kind = \"signal\"\n".to_string(),
            CommandResult::TimedOut { .. } => "kind = \"timed-out\"\n".to_string(),
            CommandResult::FailedToStart(message) => format!(
                "kind = \"failed-to-start\"\nmessage = {}\n",
                TomlWriter.quote(message)
            ),
        }
    }
}
