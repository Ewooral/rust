//! Defining enums: a type whose value is exactly ONE of a fixed set of variants.
//! Run: cargo run -p enums_and_pattern_matching --example defining_enums

// The data in these variants is only printed with `{:?}` here; reading it
// out needs `match`, which comes in the next examples.
#![allow(dead_code)]

// Simplest form: the variants are just names.
#[derive(Debug)]
enum IpAddrKind {
    V4,
    V6,
}

// Each variant can carry its own data. The variant name works like a
// constructor function: `IpAddr::V4(127, 0, 0, 1)` builds a value.
#[derive(Debug)]
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

// Variants can each hold a different shape of data: nothing, named fields
// like a struct, one value, or several values like a tuple.
#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

// Enums get methods with `impl`, exactly like structs.
impl Message {
    fn call(&self) {
        println!("called with {self:?}");
    }
}

fn route(kind: IpAddrKind) {
    println!("routing {kind:?}");
}

fn main() {
    // Variants are namespaced under the enum's name with `::`.
    route(IpAddrKind::V4);
    route(IpAddrKind::V6);

    let home = IpAddr::V4(127, 0, 0, 1);
    let loopback = IpAddr::V6(String::from("::1"));
    println!("{home:?} and {loopback:?}");

    // One type, four different shapes: a Vec<Message> can hold all of them.
    let inbox = vec![
        Message::Quit,
        Message::Move { x: 3, y: -1 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(255, 0, 128),
    ];
    for m in &inbox {
        m.call();
    }

    // An enum value is as big as its largest variant, plus a small tag that
    // records which variant it is.
    println!("a Message takes {} bytes", size_of::<Message>());
}
