//! Solution to exercise 2: write a fake by hand, and make the tests pass.
//!
//! `welcome_all` (correct, don't change it) sends a welcome mail to every
//! new user through a `Mailer`, and returns the addresses that failed. The
//! tests need a `FakeMailer` that:
//! - REMEMBERS every mail it was asked to send, as (address, subject);
//! - REFUSES the addresses it was told to refuse, with
//!   `MailError::Rejected(address)`, and doesn't record those;
//! - never touches a network.
//!
//! Implement every `todo!()`. `send` takes `&self`, yet the fake must
//! remember things: keep the list in a `RefCell<Vec<...>>`. (Why `&self`?
//! Because the real mailer doesn't need `&mut`, and the trait is shaped by
//! production code, not by tests.)
//!
//! Test: cargo test -p advanced_testing --features solutions --bin sol02_hand_written_fake

use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
enum MailError {
    Rejected(String),
}

trait Mailer {
    fn send(&self, to: &str, subject: &str) -> Result<(), MailError>;
}

/// Sends "Welcome!" to every user; returns the addresses that failed.
fn welcome_all(users: &[&str], mailer: &impl Mailer) -> Vec<String> {
    users
        .iter()
        .filter(|user| mailer.send(user, "Welcome!").is_err())
        .map(|user| user.to_string())
        .collect()
}

#[derive(Debug, Default)]
struct FakeMailer {
    sent: RefCell<Vec<(String, String)>>,
    refused: Vec<String>,
}

impl FakeMailer {
    /// A mailer that accepts every address.
    fn new() -> FakeMailer {
        FakeMailer::default()
    }

    /// The same mailer, but it refuses `address`.
    fn refusing(mut self, address: &str) -> FakeMailer {
        self.refused.push(address.to_string());
        self
    }

    /// Every mail sent so far, oldest first.
    fn sent(&self) -> Vec<(String, String)> {
        self.sent.borrow().clone()
    }
}

impl Mailer for FakeMailer {
    fn send(&self, to: &str, subject: &str) -> Result<(), MailError> {
        if self.refused.iter().any(|r| r == to) {
            return Err(MailError::Rejected(to.to_string()));
        }
        // RefCell: change the list through a shared `&self`.
        self.sent
            .borrow_mut()
            .push((to.to_string(), subject.to_string()));
        Ok(())
    }
}

fn main() {
    let mailer = FakeMailer::new().refusing("bounce@example.com");
    let failed = welcome_all(&["ama@example.com", "bounce@example.com"], &mailer);
    println!("sent: {:?}\nfailed: {failed:?}", mailer.sent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_new_user_gets_a_welcome_mail() {
        let mailer = FakeMailer::new();
        let failed = welcome_all(&["ama@example.com", "kofi@example.com"], &mailer);
        assert!(failed.is_empty());
        assert_eq!(
            mailer.sent(),
            [
                ("ama@example.com".to_string(), "Welcome!".to_string()),
                ("kofi@example.com".to_string(), "Welcome!".to_string()),
            ]
        );
    }

    #[test]
    fn refused_addresses_are_reported_and_not_recorded() {
        let mailer = FakeMailer::new().refusing("bounce@example.com");
        let failed = welcome_all(&["bounce@example.com", "esi@example.com"], &mailer);
        assert_eq!(failed, ["bounce@example.com"]);
        assert_eq!(mailer.sent().len(), 1);
    }

    #[test]
    fn the_fake_reports_why_it_refused() {
        let mailer = FakeMailer::new().refusing("x@example.com");
        assert_eq!(
            mailer.send("x@example.com", "Hi"),
            Err(MailError::Rejected("x@example.com".to_string()))
        );
    }
}
