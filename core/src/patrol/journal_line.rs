// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The journal's lines.

use alloc::format;
use alloc::string::String;
use core::fmt::Write as _;
use core::time::Duration;

use crate::model::github::discovery_error::DiscoveryError;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::roster::Roster;
use crate::model::round_step::RoundStep;
use crate::model::skip_reason::SkipReason;
use crate::model::timestamp::Timestamp;
use crate::patrol::timeout_warning::TimeoutWarning;

/// The journal's lines: one at start, one per visit (`docs/ARCHITECTURE.md`).
#[derive(Debug, Clone, Copy, Default)]
pub struct JournalLine;

impl JournalLine {
    /// `discovery repos=… added=… removed=…`, the names comma-separated, `-` for none.
    pub fn discovery(&self, repos: usize, added: &[String], removed: &[String]) -> String {
        let names = |list: &[String]| {
            if list.is_empty() {
                String::from("-")
            } else {
                list.join(",")
            }
        };
        format!(
            "discovery repos={repos} added={} removed={}",
            names(added),
            names(removed)
        )
    }

    /// `discovery failed reason="…"`.
    pub fn discovery_failed(&self, error: &DiscoveryError) -> String {
        format!("discovery failed reason={:?}", error.0)
    }

    /// The start of the journal's first line, for logs.
    pub fn startup(&self, roster: &Roster, now: Timestamp) -> String {
        format!(
            "sentinel4rust {} started at {now}: {} repositories, pause {}s, config_version {}",
            env!("CARGO_PKG_VERSION"),
            roster.repos.len(),
            roster.pause.as_secs(),
            roster.config_version
        )
    }

    /// `repo=… step=… hash=… …`.
    pub fn visit(&self, repo: &OnboardedRepo, step: &RoundStep, duration: Duration) -> String {
        let mut line = format!("repo={} step={}", repo.name, step.name());
        match step {
            RoundStep::Unchanged { head } => {
                let _ = write!(line, " hash={}", head.short());
            }
            RoundStep::Published {
                head,
                outcome,
                counts,
                new,
                gone,
            } => {
                let _ = write!(
                    line,
                    " hash={} outcome={outcome} missed={} caught={} timeout={} unviable={} new={new} gone={gone}",
                    head.short(),
                    counts.missed,
                    counts.caught,
                    counts.timeout,
                    counts.unviable,
                );
                if let Some(warning) = TimeoutWarning.of(counts) {
                    let _ = write!(line, " warning={warning}");
                }
            }
            RoundStep::BackedOff { reason }
            | RoundStep::Unreachable { reason }
            | RoundStep::Unreadable { reason } => {
                let _ = write!(line, " reason={reason:?}");
            }
            RoundStep::Skipped { reason } => {
                let _ = write!(line, " reason={}", reason.as_str());
                match reason {
                    SkipReason::PathOutsideRepo { manifest } => {
                        let _ = write!(line, " manifest={manifest}");
                    }
                    SkipReason::ToolchainMissing { message } => {
                        let _ = write!(line, " message={message:?}");
                    }
                    _ => {}
                }
            }
        }
        let _ = write!(line, " duration_s={}", duration.as_secs());
        line
    }
}
