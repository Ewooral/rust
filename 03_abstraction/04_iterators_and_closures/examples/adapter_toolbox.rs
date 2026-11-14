//! Beyond the Book: the iterator adapters you will reach for every day.
//! Run: cargo run -p iterators_and_closures --example adapter_toolbox

use std::collections::HashMap;

fn main() {
    let names = ["Ama", "Kofi", "Esi", "Yaw"];
    let scores = [72, 91, 85, 64];

    // zip: walk two sequences side by side (stops at the shorter one).
    let paired: Vec<(&str, i32)> = names.iter().copied().zip(scores).collect();
    println!("zip: {paired:?}");

    // enumerate: (index, item) pairs.
    for (i, name) in names.iter().enumerate() {
        print!("{i}:{name} ");
    }
    println!();

    // map, filter, filter_map (filter and map in one, keeping the Somes).
    let numbers: Vec<i32> = ["1", "two", "3"]
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    println!("filter_map: {numbers:?}");

    // flat_map / flatten: each item becomes many; the results are joined.
    let letters: Vec<char> = names.iter().flat_map(|n| n.chars()).take(7).collect();
    println!("flat_map: {letters:?}");
    let nested = vec![vec![1, 2], vec![], vec![3]];
    println!(
        "flatten: {:?}",
        nested.into_iter().flatten().collect::<Vec<_>>()
    );

    // fold: combine everything into one value, starting from a seed.
    // (For plain addition, `.sum()` is clearer; fold shines when the
    // result has a different type from the items, as here.)
    let report = scores
        .iter()
        .fold(String::from("scores:"), |acc, s| format!("{acc} {s}"));
    let longest = names
        .iter()
        .fold("", |best, n| if n.len() > best.len() { n } else { best });
    println!("fold: {report}; longest name {longest}");

    // scan: like fold, but yields every intermediate value (running totals).
    let running: Vec<i32> = scores
        .iter()
        .scan(0, |sum, s| {
            *sum += s;
            Some(*sum)
        })
        .collect();
    println!("scan: {running:?}");

    // windows / chunks: overlapping pairs, or fixed-size groups (on slices).
    let rises: Vec<i32> = scores.windows(2).map(|w| w[1] - w[0]).collect();
    println!("windows(2): {rises:?}");
    for group in names.chunks(2) {
        println!("chunks(2): {group:?}");
    }

    // peekable: look at the next item without taking it.
    let mut tokens = "12 + 34".chars().filter(|c| !c.is_whitespace()).peekable();
    let mut parts = Vec::new();
    while let Some(c) = tokens.next() {
        if c.is_ascii_digit() {
            let mut number = c.to_string();
            while let Some(&d) = tokens.peek() {
                if !d.is_ascii_digit() {
                    break;
                }
                number.push(d);
                tokens.next();
            }
            parts.push(number);
        } else {
            parts.push(c.to_string());
        }
    }
    println!("peekable tokenizer: {parts:?}");

    // Searching and testing.
    println!(
        "any >90: {}, all >60: {}, find >80: {:?}, position of Esi: {:?}",
        scores.iter().any(|&s| s > 90),
        scores.iter().all(|&s| s > 60),
        scores.iter().find(|&&s| s > 80),
        names.iter().position(|&n| n == "Esi"),
    );

    // min/max by a key; take_while / skip_while; chain; rev; step_by.
    let best = paired.iter().max_by_key(|(_, s)| *s).map(|(n, _)| n);
    println!("best: {best:?}");
    let ascending: Vec<i32> = [1, 2, 3, 10, 4]
        .into_iter()
        .take_while(|&x| x < 5)
        .collect();
    println!("take_while: {ascending:?}");
    let both: Vec<i32> = (1..3).chain(7..9).rev().collect();
    println!("chain + rev: {both:?}");
    println!("step_by: {:?}", (0..20).step_by(5).collect::<Vec<_>>());

    // collect into many shapes: Vec, String, HashMap, Result.
    let lookup: HashMap<&str, i32> = paired.iter().copied().collect();
    let initials: String = names
        .iter()
        .map(|n| n.chars().next().unwrap_or('?'))
        .collect();
    let all_ok: Result<Vec<i32>, _> = ["1", "2"].iter().map(|s| s.parse::<i32>()).collect();
    let one_bad: Result<Vec<i32>, _> = ["1", "x"].iter().map(|s| s.parse::<i32>()).collect();
    println!(
        "HashMap: Kofi -> {:?}; String: {initials}; {all_ok:?}; {}",
        lookup.get("Kofi"),
        one_bad.is_err()
    );

    // partition: split into two collections by a test.
    let (passed, failed): (Vec<i32>, Vec<i32>) = scores.iter().partition(|&&s| s >= 70);
    println!("partition: passed {passed:?}, failed {failed:?}");
}
