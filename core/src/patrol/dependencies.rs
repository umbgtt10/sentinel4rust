// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Everything the sentinel talks to.

use alloc::rc::Rc;

use crate::traits::clock::Clock;
use crate::traits::journal::Journal;
use crate::traits::mutation_runner::MutationRunner;
use crate::traits::pause::Pause;
use crate::traits::repo_discovery::RepoDiscovery;
use crate::traits::repo_source::RepoSource;
use crate::traits::result_store::ResultStore;

/// Everything the sentinel talks to, injected once. Shared, so one implementation (`GitCli`, or a test's
/// fake) can fill several roles.
pub struct Dependencies {
    /// Fetches and checks out repositories.
    pub source: Rc<dyn RepoSource>,
    /// Runs cargo-mutants.
    pub runner: Rc<dyn MutationRunner>,
    /// Publishes to the results repository.
    pub store: Rc<dyn ResultStore>,
    /// The time.
    pub clock: Rc<dyn Clock>,
    /// Sleeping.
    pub pause: Rc<dyn Pause>,
    /// One line per visit.
    pub journal: Rc<dyn Journal>,
    /// Finds the repositories to watch; `None`: the roster's are all there is.
    pub discovery: Option<Rc<dyn RepoDiscovery>>,
}
