//! Defining a struct, creating instances, reading and changing fields.
//! Run: cargo run -p structs --example defining_and_instantiating

// The definition is the blank form: field names and their types.
#[derive(Debug)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

/// Field init shorthand: when a parameter has the same name as a field,
/// write `email` instead of `email: email`.
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,
        email,
        sign_in_count: 1,
    }
}

fn main() {
    // An instance is a filled-in form. Field order does not matter.
    let user1 = User {
        email: String::from("someone@example.com"),
        username: String::from("someusername123"),
        active: true,
        sign_in_count: 1,
    };
    println!("{} <{}>", user1.username, user1.email);

    // Mutability belongs to the whole instance, never to single fields.
    let mut user2 = build_user(String::from("ferris@example.com"), String::from("ferris"));
    user2.email = String::from("crab@example.com");
    user2.sign_in_count += 1;
    println!("{user2:?}");

    // user1.email = String::from("x"); // E0594: `user1` is not declared as mutable

    // The struct owns its Strings, so it is valid as long as the struct is.
    // Storing `&str` fields instead needs lifetimes (lesson 03/01).
    println!(
        "active: {}, sign-ins: {}",
        user1.active, user1.sign_in_count
    );
}
