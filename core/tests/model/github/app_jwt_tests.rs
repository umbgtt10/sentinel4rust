// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::app_jwt::AppJwt;
use sentinel4rust::model::github::base64::Base64;
use sentinel4rust::model::timestamp::Timestamp;

#[test]
fn signing_input_is_the_rs256_header_and_claims_a_minute_back_nine_ahead() {
    // Arrange
    let jwt = AppJwt::new(123);

    // Act
    let input = jwt.signing_input(Timestamp::from_unix(1_700_000_000));

    // Assert
    let claims = Base64.url(br#"{"iat":1699999940,"exp":1700000540,"iss":"123"}"#);
    assert_eq!(
        input,
        format!("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.{claims}")
    );
}

#[test]
fn token_appends_the_signature_in_the_url_alphabet() {
    // Arrange
    let jwt = AppJwt::new(123);

    // Act
    let token = jwt.token("head.claims", &[0xfb, 0xff]);

    // Assert
    assert_eq!(token, "head.claims.-_8");
}
