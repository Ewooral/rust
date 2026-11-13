//! Struct update syntax: build a new instance from an old one.
//! Run: cargo run -p structs --example struct_update_syntax

#[derive(Debug)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

fn main() {
    let user1 = User {
        active: true,
        username: String::from("ferris"),
        email: String::from("ferris@example.com"),
        sign_in_count: 1,
    };

    // `..user1` fills every field we did not list, taking it from user1.
    // `..` must come last.
    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
    println!("{user2:?}");

    // `..user1` works like assignment, so it MOVES `username` (a String)
    // out of user1. The Copy fields (`active`, `sign_in_count`) are copied.
    // println!("{}", user1.username); // E0382: borrow of moved value: `user1.username`
    println!("still usable: {} {}", user1.email, user1.sign_in_count);

    // If every moved field is given a fresh value, only Copy fields are
    // taken, and user1 stays completely whole.
    let user3 = User {
        username: String::from("corro"),
        email: String::from("corro@example.com"),
        ..user2
    };
    println!("{user3:?}");
    println!("user2 is still whole: {user2:?}");
    println!("{} active: {}", user3.username, user3.active);
}
