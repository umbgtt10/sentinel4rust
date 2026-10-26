// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! git's answers read: the refs `ls-remote` listed, a failure that means the repository is unreachable, and a
//! manifest's `path` dependencies that leave the repository (`ADR-FiltersLiveInEachRepository`).

pub mod outside_paths;
pub mod remote_listing;
pub mod unreachable_classifier;
