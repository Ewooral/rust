//! The macros used the way another crate uses them: only through
//! `macros_in_depth`'s public API. This checks that the generated code's
//! `::macros_in_depth::…` paths and the `$crate::…` paths work from outside.
//!
//! Run: cargo test -p macros_in_depth --test as_a_user

use std::cell::Cell;

use macros_in_depth::{Settings, SettingsError, ensure, memoize, palette, settings_map};

/// A job queue's settings.
#[derive(Debug, Settings)]
struct Queue {
    /// The queue's name.
    name: String,
    /// How many jobs run at once.
    #[setting(rename = "max-jobs", default = "2")]
    max_jobs: u8,
    /// Seconds before a job is given up.
    timeout: Option<f64>,
}

#[test]
fn the_derive_works_from_another_crate() {
    let queue = Queue::from_text("name = emails\ntimeout = 1.5").unwrap();
    assert_eq!(queue.name, "emails");
    assert_eq!(queue.max_jobs, 2);
    assert_eq!(queue.timeout, Some(1.5));
    assert_eq!(Queue::fields()[1].key, "max-jobs");
    assert!(Queue::help().contains("How many jobs run at once. [default: 2]"));
}

fn validate(queue: &Queue) -> Result<(), SettingsError> {
    ensure!(
        queue.max_jobs > 0,
        "{} needs at least one job slot",
        queue.name
    );
    Ok(())
}

#[test]
fn the_macro_rules_macros_work_from_another_crate() {
    let map = settings_map! { "name" => "reports", "max-jobs" => 0 };
    let queue = Queue::from_map_checked(&map).unwrap();
    assert_eq!(
        validate(&queue).unwrap_err().to_string(),
        "reports needs at least one job slot"
    );
}

palette! {
    pub(crate) mod brand {
        ink = "#1a1a2e",
        paper = "#fffdf7",
    }
}

thread_local! {
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

#[memoize(capacity = 2)]
fn shout(word: String) -> String {
    CALLS.with(|calls| calls.set(calls.get() + 1));
    word.to_uppercase()
}

#[test]
fn palette_and_memoize_work_from_another_crate() {
    assert_eq!(brand::INK.to_string(), "#1a1a2e");
    assert_eq!(brand::ALL.len(), 2);

    assert_eq!(shout("a".into()), "A");
    assert_eq!(shout("a".into()), "A"); // from the cache
    assert_eq!(shout("b".into()), "B"); // the cache now holds 2
    assert_eq!(shout("c".into()), "C"); // full: the cache is emptied first
    assert_eq!(shout("a".into()), "A"); // so "a" is computed again
    assert_eq!(CALLS.with(Cell::get), 4);
}
