//! The data model.

mod commit_hash;
mod counts;
mod mutant_key;
mod outcome;
mod record;
mod roster;
mod timestamp;

pub use commit_hash::CommitHash;
pub use counts::MutantCounts;
pub use mutant_key::{MutantKey, new_and_gone};
pub use outcome::{MutationRun, OutcomeKind, RoundStep, RunOutcome, SkipReason};
pub use record::{BRANCH_README, KIND_FULL, Publication, RecordError, RunRecord, SCHEMA};
pub use roster::{OnboardedRepo, Roster, RosterError};
pub use timestamp::Timestamp;
