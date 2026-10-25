// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::quote_state::QuoteState;

#[test]
fn after_a_backslash_in_a_basic_string_the_next_quote_is_text() {
    // Arrange & Act
    let state = QuoteState::Basic.after('\\').after('"');

    // Assert
    assert_eq!(state, QuoteState::Basic);
}

#[test]
fn after_a_quote_in_code_a_string_opens_and_the_same_quote_closes_it() {
    // Arrange & Act & Assert
    assert_eq!(QuoteState::Code.after('"'), QuoteState::Basic);
    assert_eq!(QuoteState::Code.after('\''), QuoteState::Literal);
    assert_eq!(QuoteState::Basic.after('"'), QuoteState::Code);
    assert_eq!(QuoteState::Literal.after('\''), QuoteState::Code);
    assert_eq!(QuoteState::Literal.after('"'), QuoteState::Literal);
}

#[test]
fn is_code_holds_only_outside_strings() {
    // Arrange & Act & Assert
    assert!(QuoteState::Code.is_code());
    assert!(!QuoteState::Basic.is_code());
    assert!(!QuoteState::BasicEscape.is_code());
    assert!(!QuoteState::Literal.is_code());
}
