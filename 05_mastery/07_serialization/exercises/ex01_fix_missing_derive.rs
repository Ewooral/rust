//! Exercise 1: make it compile (E0277).
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
//! Run: cargo run -p serialization --features exercises --bin ex01_fix_missing_derive

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Timetable {
    class: String,
    lessons: Vec<Lesson>,
}

#[derive(Debug, PartialEq)]
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
