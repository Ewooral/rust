//! `HashMap<K, V>`: look values up by a key instead of by position.
//! Run: cargo run -p collections --example hash_maps

// HashMap is not in the prelude: bring it into scope.
use std::collections::HashMap;

fn main() {
    // Creating and inserting
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    // Accessing: `get` returns Option<&V>.
    let team_name = String::from("Blue");
    let score = scores.get(&team_name).copied().unwrap_or(0);
    println!(
        "Blue: {score}, Red: {}",
        scores.get("Red").copied().unwrap_or(0)
    );

    // Iterating: the order is arbitrary and can change between runs.
    for (key, value) in &scores {
        println!("{key}: {value}");
    }

    // Ownership: owned values like String are MOVED into the map.
    let field_name = String::from("Favorite color");
    let field_value = String::from("Blue");
    let mut map = HashMap::new();
    map.insert(field_name, field_value);
    // println!("{field_name}"); // E0382: borrow of moved value: `field_name`
    println!("{map:?}");

    // Updating 1: overwrite. Inserting the same key again replaces the value.
    scores.insert(String::from("Blue"), 25);

    // Updating 2: add only if the key is missing, with the entry API.
    scores.entry(String::from("Yellow")).or_insert(50); // already there: no change
    scores.entry(String::from("Red")).or_insert(0); // missing: inserted
    let mut sorted: Vec<_> = scores.iter().collect();
    sorted.sort(); // sort to print in a predictable order
    println!("{sorted:?}");

    // Updating 3: change a value based on the old one.
    // `or_insert` returns `&mut V`, so we can add to it in place.
    let text = "hello world wonderful world";
    let mut word_counts = HashMap::new();
    for word in text.split_whitespace() {
        let count = word_counts.entry(word).or_insert(0);
        *count += 1;
    }
    let mut counts: Vec<_> = word_counts.into_iter().collect();
    counts.sort();
    println!("{counts:?}");

    // Hashing: the default hasher (SipHash 1-3) is slower than some, but
    // resists attacks that feed a map keys chosen to collide.
}
