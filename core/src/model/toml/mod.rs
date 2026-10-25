// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A small subset of TOML: enough for the roster and `summary.toml`, so the tool needs no dependencies.
//!
//! Supported: comments, bare and quoted keys, basic and literal strings, integers, booleans, and arrays of
//! tables (`[[repo]]`). Anything else is rejected with the line number.

pub mod basic_string;
pub mod comment_stripper;
pub mod document;
pub mod document_builder;
pub mod quote_state;
pub mod table;
pub mod toml_error;
pub mod toml_parser;
pub mod toml_writer;
pub mod value;
