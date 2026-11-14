//! Solution to exercise 2: make it compile (E0277), WITHOUT logging the password.
//!
//! `#[instrument]` records every argument of the function as a span field,
//! using `Debug`. `Password` deliberately has no `Debug`, so this doesn't
//! compile. The compiler suggests deriving `Debug`... which would compile
//! and then write every user's password into the logs. (The test checks.)
//!
//! Tell `#[instrument]` to leave the password out. Look up `skip(...)`.
//!
//! Test: cargo test -p observability --features solutions --bin sol02_fix_instrument_secret

use tracing::{info, instrument};

struct Password(String);

// skip(password): never record it. Deriving Debug would have compiled
// and leaked every password into the logs.
#[instrument(skip(password))]
fn log_in(user: &str, password: &Password) -> bool {
    let ok = password.0.len() >= 8;
    info!(ok, "login attempt");
    ok
}

fn main() {
    tracing_subscriber::fmt().init();
    log_in("ama", &Password("hunter2-but-longer".to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::capture::CapturedLogs;

    #[test]
    fn logs_the_user_but_never_the_password() {
        let logs = CapturedLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            assert!(log_in("ama", &Password("s3cret-s3cret".to_string())));
        });
        let text = logs.contents();
        assert!(text.contains("log_in{user=\"ama\"}"), "{text}");
        assert!(
            !text.contains("s3cret"),
            "the password leaked into the logs:\n{text}"
        );
    }
}
