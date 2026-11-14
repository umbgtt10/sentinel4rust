// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! How far each repository's record is behind its `main`.

use alloc::format;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::time::Duration;

use crate::model::branch::record_currency::RecordCurrency;
use crate::model::branch::record_state::RecordState;
use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::snapshot::Snapshot;
use crate::model::sync_error::SyncError;
use crate::patrol::status_line::StatusLine;
use crate::patrol::timeout_warning::TimeoutWarning;
use crate::traits::clock::Clock;
use crate::traits::repo_source::RepoSource;

/// A read-only pass over the roster: fetches each repository, runs nothing, pushes nothing, and says whether its
/// record in the results repository answers for `main` and, when not, how long `main`'s head has waited for one -- and warns of a
/// record whose timeouts outnumber its caught mutants, which needs a look whether current or not.
pub struct StatusReport {
    source: Rc<dyn RepoSource>,
    clock: Rc<dyn Clock>,
    currency: RecordCurrency,
    stale_after: Duration,
}

impl StatusReport {
    /// Fetches through `source`, takes a record as current when it names `main`'s head under `config_version`
    /// -- whichever cargo-mutants made it: a status machine need not have it installed --, and calls a repository
    /// stale when `main`'s head has waited longer than `stale_after`.
    pub fn new(
        source: Rc<dyn RepoSource>,
        clock: Rc<dyn Clock>,
        config_version: i64,
        stale_after: Duration,
    ) -> Self {
        Self {
            source,
            clock,
            currency: RecordCurrency::new(config_version, None),
            stale_after,
        }
    }

    /// One line per repository, in roster order.
    pub fn check(&self, repos: &[OnboardedRepo]) -> Vec<StatusLine> {
        repos.iter().map(|repo| self.line(repo)).collect()
    }

    fn judged(&self, repo: &OnboardedRepo, snapshot: &Snapshot) -> StatusLine {
        let main = snapshot.head.short();
        match (&snapshot.record, &snapshot.readiness) {
            (RecordState::Unreadable(error), _) => self.waiting(
                repo,
                &snapshot.head,
                &format!(
                    "state=unreadable main={main} reason={:?}",
                    error.to_string()
                ),
            ),
            (RecordState::Present(record), _)
                if self.currency.is_current(record, &snapshot.head) =>
            {
                Self::warned(
                    StatusLine {
                        text: format!(
                            "repo={} state=current main={main} record={}",
                            repo.name,
                            record.commit.short()
                        ),
                        needs_attention: false,
                    },
                    record,
                )
            }
            (_, Err(reason)) => StatusLine {
                text: format!(
                    "repo={} state=skipped main={main} reason={}",
                    repo.name,
                    reason.as_str()
                ),
                needs_attention: false,
            },
            (RecordState::Present(record), Ok(())) => Self::warned(
                self.waiting(
                    repo,
                    &snapshot.head,
                    &format!("state=behind main={main} record={}", record.commit.short()),
                ),
                record,
            ),
            (RecordState::Absent, Ok(())) => self.waiting(
                repo,
                &snapshot.head,
                &format!("state=no-record main={main}"),
            ),
        }
    }

    fn line(&self, repo: &OnboardedRepo) -> StatusLine {
        match self.source.sync(repo) {
            Ok(snapshot) => self.judged(repo, &snapshot),
            Err(error) => {
                let reason = match error {
                    SyncError::Unreachable(reason) | SyncError::Transient(reason) => reason,
                    SyncError::NoBranch => "no-branch".to_string(),
                };
                StatusLine {
                    text: format!("repo={} state=unknown reason={reason:?}", repo.name),
                    needs_attention: true,
                }
            }
        }
    }

    fn waiting(&self, repo: &OnboardedRepo, head: &CommitHash, state: &str) -> StatusLine {
        match self.source.committed_at(repo, head) {
            Some(committed) => {
                let waited = self.clock.now().since(committed);
                StatusLine {
                    text: format!("repo={} {state} waited_s={}", repo.name, waited.as_secs()),
                    needs_attention: waited > self.stale_after,
                }
            }
            None => StatusLine {
                text: format!("repo={} {state} waited_s=unknown", repo.name),
                needs_attention: true,
            },
        }
    }

    // `line` with the record's warning, if it calls for one, which then needs attention.
    fn warned(line: StatusLine, record: &RunRecord) -> StatusLine {
        match TimeoutWarning.of(&record.counts) {
            Some(warning) => StatusLine {
                text: format!("{} warning={warning}", line.text),
                needs_attention: true,
            },
            None => line,
        }
    }
}
