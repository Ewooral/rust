//! Solution to exercise 2: make it compile, WITHOUT `.clone()`.
//!
//! `..p1` moved something out of `p1` (E0382). Give `p2` its own values for
//! the fields that would be moved, so only the Copy fields come from `p1`
//! and `p1` stays whole.
//!
//! Run: cargo run -p structs --features solutions --bin sol02_fix_update_move

#[derive(Debug)]
struct Profile {
    username: String,
    country: String,
    age: u8,
    verified: bool,
}

fn main() {
    let p1 = Profile {
        username: String::from("ferris"),
        country: String::from("Ghana"),
        age: 9,
        verified: true,
    };
    let p2 = Profile {
        username: String::from("corro"),
        country: String::from("Peru"),
        ..p1
    };
    println!("{p2:?}");
    println!("p2 copied age {} and verified {}", p2.age, p2.verified);
    println!("{} is from {}", p1.username, p1.country);
}
