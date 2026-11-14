//! Access tokens: JSON Web Tokens signed with HMAC-SHA256 (`HS256`).
//!
//! A JWT is three base64url parts joined by dots: `header.claims.signature`.
//! Anyone can READ the claims (they are not encrypted, so never put secrets
//! in them), but only someone with the server's secret can make a signature
//! that matches, so the server can trust the claims without a database lookup.
//!
//! `jsonwebtoken` 12 needs a crypto backend; this crate enables the pure-Rust
//! `rust_crypto` one (the other, `aws_lc_rs`, builds C and assembly code).

use std::time::Duration;

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};

use crate::config::Secret;
use crate::domain::Role;
use crate::error::AppError;

/// What the token says about its holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the user id (JWT says it's a string).
    pub sub: String,
    /// The user's role when the token was issued.
    pub role: Role,
    /// Issuer: who made the token.
    pub iss: String,
    /// Issued at (seconds since 1970).
    pub iat: u64,
    /// Expires at (seconds since 1970).
    pub exp: u64,
}

/// Issues and checks tokens with one secret.
#[derive(Clone)]
pub struct Tokens {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    issuer: String,
    ttl: Duration,
}

impl Tokens {
    /// Keys from the secret; tokens live for `ttl`.
    pub fn new(secret: &Secret, issuer: &str, ttl: Duration) -> Result<Tokens, AppError> {
        let bytes = secret.expose().as_bytes();
        let key_error =
            |err: jsonwebtoken::errors::Error| AppError::Internal(format!("JWT key: {err}"));
        // Accept ONLY HS256: a server that lets the token's header choose the
        // algorithm can be fooled ("alg confusion"). Require the claims we use.
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[issuer]);
        validation.set_required_spec_claims(&["exp", "iss", "sub"]);
        validation.leeway = 5; // seconds of clock difference to tolerate
        Ok(Tokens {
            encoding: EncodingKey::from_secret(bytes).map_err(key_error)?,
            decoding: DecodingKey::from_secret(bytes).map_err(key_error)?,
            validation,
            issuer: issuer.to_string(),
            ttl,
        })
    }

    /// How long new tokens live.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// A token for this user, valid from now for `ttl`.
    pub fn issue(&self, user_id: i64, role: Role) -> Result<String, AppError> {
        self.issue_at(user_id, role, get_current_timestamp())
    }

    /// A token as if issued at `now` (seconds since 1970): handy in tests.
    pub fn issue_at(&self, user_id: i64, role: Role, now: u64) -> Result<String, AppError> {
        let claims = Claims {
            sub: user_id.to_string(),
            role,
            iss: self.issuer.clone(),
            iat: now,
            exp: now + self.ttl.as_secs(),
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .map_err(|err| AppError::Internal(format!("signing a token: {err}")))
    }

    /// Checks the signature, algorithm, issuer and expiry, and returns the claims.
    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding, &self.validation)
            .map(|data| data.claims)
            .map_err(|err| match err.kind() {
                ErrorKind::ExpiredSignature => AppError::Unauthorized("the token has expired"),
                _ => AppError::Unauthorized("the token is not valid"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(secret: &str) -> Tokens {
        Tokens::new(&Secret::new(secret), "test", Duration::from_secs(60)).unwrap()
    }

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn a_token_round_trips() {
        let tokens = tokens(SECRET);
        let token = tokens.issue(7, Role::Admin).unwrap();
        assert_eq!(token.split('.').count(), 3);
        let claims = tokens.verify(&token).unwrap();
        assert_eq!((claims.sub.as_str(), claims.role), ("7", Role::Admin));
        assert_eq!(claims.exp - claims.iat, 60);
    }

    #[test]
    fn expired_tokens_are_refused() {
        let tokens = tokens(SECRET);
        let an_hour_ago = get_current_timestamp() - 3600;
        let old = tokens.issue_at(7, Role::Member, an_hour_ago).unwrap();
        let err = tokens.verify(&old).unwrap_err();
        assert_eq!(err.to_string(), "the token has expired");
    }

    #[test]
    fn a_different_secret_or_a_changed_token_is_refused() {
        let token = tokens(SECRET).issue(7, Role::Member).unwrap();
        assert!(
            tokens("another-secret-another-secret-12")
                .verify(&token)
                .is_err()
        );

        // Change one character of the claims: the signature no longer matches.
        let (head, rest) = token.split_once('.').unwrap();
        let (claims, signature) = rest.split_once('.').unwrap();
        let mut forged = claims.to_string();
        let last = forged.pop().unwrap();
        forged.push(if last == 'A' { 'B' } else { 'A' });
        let forged = format!("{head}.{forged}.{signature}");
        assert!(tokens(SECRET).verify(&forged).is_err());
        assert!(tokens(SECRET).verify("not.a.token").is_err());
    }
}
