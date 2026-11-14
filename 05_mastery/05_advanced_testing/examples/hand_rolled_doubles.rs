//! Test doubles by hand: dummy, stub, spy, fake and mock, with no crates.
//!
//! Run:  cargo run -p advanced_testing --example hand_rolled_doubles
//! Test: cargo test -p advanced_testing --example hand_rolled_doubles

/// The dependency: something that can send a message to a user.
trait Notifier {
    fn notify(&self, user: &str, message: &str) -> Result<(), String>;
}

/// Something else the code needs, but not every test cares about.
trait AuditLog {
    fn record(&self, event: &str);
}

struct Invoice {
    customer: String,
    days_overdue: u32,
}

/// The code under test. Its dependencies come IN as parameters
/// (dependency injection), so a test can pass a double instead.
fn remind_overdue(invoices: &[Invoice], notifier: &dyn Notifier, audit: &dyn AuditLog) -> usize {
    let mut sent = 0;
    for invoice in invoices.iter().filter(|i| i.days_overdue > 30) {
        let message = format!("Your invoice is {} days overdue", invoice.days_overdue);
        match notifier.notify(&invoice.customer, &message) {
            Ok(()) => sent += 1,
            Err(e) => audit.record(&format!("could not remind {}: {e}", invoice.customer)),
        }
    }
    sent
}

// --- The real implementations (what production would use) ---------------

struct PrintNotifier;
impl Notifier for PrintNotifier {
    fn notify(&self, user: &str, message: &str) -> Result<(), String> {
        println!("  to {user}: {message}");
        Ok(())
    }
}

struct PrintAudit;
impl AuditLog for PrintAudit {
    fn record(&self, event: &str) {
        println!("  audit: {event}");
    }
}

fn main() {
    let invoices = [
        Invoice {
            customer: "ama".into(),
            days_overdue: 45,
        },
        Invoice {
            customer: "kofi".into(),
            days_overdue: 3,
        },
    ];
    let sent = remind_overdue(&invoices, &PrintNotifier, &PrintAudit);
    println!("{sent} reminder(s) sent");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashSet;

    fn invoice(customer: &str, days_overdue: u32) -> Invoice {
        Invoice {
            customer: customer.to_string(),
            days_overdue,
        }
    }

    // DUMMY: passed because the signature needs it, never actually used.
    struct DummyAudit;
    impl AuditLog for DummyAudit {
        fn record(&self, _: &str) {
            panic!("the audit log should not be used in this test");
        }
    }

    // STUB: gives canned answers, records nothing.
    struct AlwaysOk;
    impl Notifier for AlwaysOk {
        fn notify(&self, _: &str, _: &str) -> Result<(), String> {
            Ok(())
        }
    }

    // SPY: a stub that also remembers how it was called. `&self` methods
    // can't change fields, so the record lives in a RefCell.
    #[derive(Default)]
    struct SpyNotifier {
        calls: RefCell<Vec<(String, String)>>,
    }
    impl Notifier for SpyNotifier {
        fn notify(&self, user: &str, message: &str) -> Result<(), String> {
            self.calls
                .borrow_mut()
                .push((user.to_string(), message.to_string()));
            Ok(())
        }
    }

    #[derive(Default)]
    struct SpyAudit {
        events: RefCell<Vec<String>>,
    }
    impl AuditLog for SpyAudit {
        fn record(&self, event: &str) {
            self.events.borrow_mut().push(event.to_string());
        }
    }

    // FAKE: a real, working, simplified implementation (an in-memory list of
    // known users), with real behaviour such as rejecting unknown users.
    struct FakeNotifier {
        known_users: HashSet<String>,
    }
    impl Notifier for FakeNotifier {
        fn notify(&self, user: &str, _: &str) -> Result<(), String> {
            if self.known_users.contains(user) {
                Ok(())
            } else {
                Err("no such user".to_string())
            }
        }
    }

    // MOCK: knows in advance which calls it EXPECTS, and fails the test if
    // they don't happen (checked when it's dropped at the end of the test).
    struct MockNotifier {
        expected_users: RefCell<Vec<String>>,
    }
    impl Notifier for MockNotifier {
        fn notify(&self, user: &str, _: &str) -> Result<(), String> {
            let mut expected = self.expected_users.borrow_mut();
            assert_eq!(
                expected.first().map(String::as_str),
                Some(user),
                "unexpected call"
            );
            expected.remove(0);
            Ok(())
        }
    }
    impl Drop for MockNotifier {
        fn drop(&mut self) {
            if !std::thread::panicking() {
                assert!(
                    self.expected_users.borrow().is_empty(),
                    "expected calls never happened: {:?}",
                    self.expected_users.borrow()
                );
            }
        }
    }

    #[test]
    fn counts_the_reminders_sent() {
        let invoices = [invoice("ama", 45), invoice("kofi", 3), invoice("esi", 31)];
        assert_eq!(remind_overdue(&invoices, &AlwaysOk, &DummyAudit), 2);
    }

    #[test]
    fn reminds_only_invoices_more_than_30_days_overdue() {
        let spy = SpyNotifier::default();
        remind_overdue(&[invoice("ama", 30), invoice("esi", 31)], &spy, &DummyAudit);
        let calls = spy.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0],
            (
                "esi".to_string(),
                "Your invoice is 31 days overdue".to_string()
            )
        );
    }

    #[test]
    fn failures_are_written_to_the_audit_log() {
        let fake = FakeNotifier {
            known_users: HashSet::from(["ama".to_string()]),
        };
        let audit = SpyAudit::default();
        let sent = remind_overdue(&[invoice("ama", 40), invoice("ghost", 40)], &fake, &audit);
        assert_eq!(sent, 1);
        assert_eq!(
            *audit.events.borrow(),
            ["could not remind ghost: no such user"]
        );
    }

    #[test]
    fn reminds_in_invoice_order() {
        let mock = MockNotifier {
            expected_users: RefCell::new(vec!["ama".into(), "esi".into()]),
        };
        remind_overdue(
            &[invoice("ama", 40), invoice("esi", 50)],
            &mock,
            &DummyAudit,
        );
    } // <- mock dropped here: it checks that both expected calls happened
}
