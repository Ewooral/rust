//! Solution to exercise 1: make it compile (E0277).
//!
//! A school saves its timetable as JSON. `Timetable` derives Serialize and
//! Deserialize, but the derive only writes code that calls `serialize` and
//! `deserialize` on every FIELD. One of the field types can't do that yet.
//!
//! Read the error: which type is missing which trait? Fix it with one line.
//! (Don't write the impls by hand: derive them.) Then read the error again:
//! why does it say `Serialize` AND `Deserialize`, and why is it reported on
//! the field, not on `Lesson`?
//!
//! Answer: `to_string_pretty` needs `Timetable: Serialize` and `from_str`
//! needs `Timetable: Deserialize`. The derived impls call the trait on every
//! field, so both fail at the field whose type (`Vec<Lesson>`) lacks them.
//! `Vec<T>` is Serialize only when `T` is, so the root cause is `Lesson`.
//!
//! Run: cargo run -p serialization --features solutions --bin sol01_fix_missing_derive

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Timetable {
    class: String,
    lessons: Vec<Lesson>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Lesson {
    subject: String,
    room: u16,
}

fn main() {
    let monday = Timetable {
        class: "4B".to_string(),
        lessons: vec![
            Lesson {
                subject: "Maths".to_string(),
                room: 12,
            },
            Lesson {
                subject: "Art".to_string(),
                room: 3,
            },
        ],
    };
    let json = serde_json::to_string_pretty(&monday).unwrap();
    println!("{json}");
    let back: Timetable = serde_json::from_str(&json).unwrap();
    assert_eq!(back, monday);
    println!("round trip OK");
}
