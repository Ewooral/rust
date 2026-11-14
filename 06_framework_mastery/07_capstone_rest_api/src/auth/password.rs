//! Password hashing with Argon2id (pure Rust, the `argon2` crate).
//!
//! Never store a password, or a fast hash of one (SHA-256 can be tried
//! billions of times a second). Argon2 is *deliberately* slow and memory
//! hungry, and every hash has its own random salt, so identical passwords
//! give different hashes. The result is a self-describing "PHC string":
//!
//! `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`

use std::sync::Arc;

use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::{Argon2, Params};

use crate::error::AppError;

/// Hashes and checks passwords with fixed Argon2id costs.
#[derive(Clone)]
pub struct Passwords {
    params: Params,
    /// A real hash of a random string, for `verify_dummy`.
    dummy_hash: Arc<str>,
}

impl Passwords {
    /// `memory_kib` and `iterations` are Argon2's `m` and `t`. OWASP's
    /// suggestion, and this crate's default, is 19 MiB and 2 passes.
    pub fn new(memory_kib: u32, iterations: u32) -> Result<Passwords, argon2::Error> {
        let params = Params::new(memory_kib, iterations, 1, None)?;
        let mut passwords = Passwords {
            params,
            dummy_hash: Arc::from(""),
        };
        let dummy = passwords
            .hash("not a real password, just for timing")
            .map_err(|_| argon2::Error::OutputTooShort)?;
        passwords.dummy_hash = Arc::from(dummy.as_str());
        Ok(passwords)
    }

    fn hasher(&self) -> Argon2<'static> {
        Argon2::from(self.params.clone())
    }

    /// Hashes a password with a fresh random salt. Slow on purpose: call it
    /// through `tokio::task::spawn_blocking` from async code (see `service`).
    pub fn hash(&self, password: &str) -> Result<String, AppError> {
        self.hasher()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|err| AppError::Internal(format!("hashing a password: {err}")))
    }

    /// True if `password` matches the stored PHC string. The costs and salt
    /// come from the string itself, so old hashes still verify after the
    /// costs are raised.
    pub fn verify(&self, password: &str, phc: &str) -> bool {
        self.hasher()
            .verify_password(password.as_bytes(), phc)
            .is_ok()
    }

    /// Spends the same time as a real `verify`, for logins with an unknown
    /// e-mail address: otherwise the quick "no such user" answer would tell
    /// an attacker which addresses have accounts.
    pub fn verify_dummy(&self, password: &str) {
        let _ = self.verify(password, &self.dummy_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cheap() -> Passwords {
        Passwords::new(Params::MIN_M_COST, 1).unwrap()
    }

    #[test]
    fn a_hash_verifies_only_its_own_password() {
        let passwords = cheap();
        let hash = passwords.hash("correct horse battery").unwrap();
        assert!(hash.starts_with("$argon2id$v=19$m=8,t=1,p=1$"), "{hash}");
        assert!(passwords.verify("correct horse battery", &hash));
        assert!(!passwords.verify("correct horse batterY", &hash));
        assert!(!passwords.verify("anything", "not a phc string"));
    }

    #[test]
    fn the_same_password_gets_a_different_salt_each_time() {
        let passwords = cheap();
        let first = passwords.hash("same password").unwrap();
        let second = passwords.hash("same password").unwrap();
        assert_ne!(first, second);
        assert!(passwords.verify("same password", &first));
        assert!(passwords.verify("same password", &second));
    }

    #[test]
    fn hashes_made_with_other_costs_still_verify() {
        let old = Passwords::new(16, 2).unwrap().hash("long-lived").unwrap();
        assert!(cheap().verify("long-lived", &old));
    }

    #[test]
    fn too_little_memory_is_refused() {
        assert!(Passwords::new(1, 1).is_err());
    }
}
