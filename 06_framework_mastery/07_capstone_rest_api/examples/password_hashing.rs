//! Password hashing with Argon2id: salts, PHC strings, costs and timing.
//! Run: cargo run -p capstone_rest_api --example password_hashing
//!
//! (A debug build is many times slower than `--release` at this.)

use std::time::Instant;

use capstone_rest_api::auth::Passwords;

fn main() {
    let password = "correct horse battery staple";

    // The production cost (19 MiB, 2 passes) and the test cost (8 KiB, 1 pass).
    for (memory_kib, iterations, label) in [(19 * 1024, 2, "production"), (8, 1, "tests")] {
        let passwords = Passwords::new(memory_kib, iterations).expect("valid parameters");
        let started = Instant::now();
        let hash = passwords.hash(password).expect("hashing works");
        let took = started.elapsed();
        println!("{label:<10} cost: hashed in {took:>10.2?}");
        println!("           {hash}");
    }

    let passwords = Passwords::new(8, 1).expect("valid parameters");
    let first = passwords.hash(password).expect("hashing works");
    let second = passwords.hash(password).expect("hashing works");
    println!("\nsame password, different salts: {}", first != second);
    println!(
        "right password verifies:        {}",
        passwords.verify(password, &first)
    );
    println!(
        "wrong password verifies:        {}",
        passwords.verify("correct horse battery stapler", &first)
    );

    // The PHC string carries everything needed to check it later.
    let parts: Vec<&str> = first.split('$').collect();
    println!("\nthe PHC string, part by part:");
    for (name, part) in ["algorithm", "version", "costs", "salt", "hash"]
        .iter()
        .zip(parts.iter().skip(1))
    {
        println!("  {name:<9} {part}");
    }
}
