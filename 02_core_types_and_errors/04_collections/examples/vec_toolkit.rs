//! Beyond the Book: everyday Vec methods, and capacity.
//! Run: cargo run -p collections --example vec_toolkit

/// Two pointers: in a SORTED slice, find two values that add up to `target`.
/// One index starts at each end and they walk towards each other: O(n),
/// instead of trying every pair, which is O(n * n).
fn pair_with_sum(sorted: &[i32], target: i32) -> Option<(i32, i32)> {
    if sorted.is_empty() {
        return None;
    }
    let (mut left, mut right) = (0, sorted.len() - 1);
    while left < right {
        let sum = sorted[left] + sorted[right];
        match sum.cmp(&target) {
            std::cmp::Ordering::Equal => return Some((sorted[left], sorted[right])),
            std::cmp::Ordering::Less => left += 1, // need bigger: move left up
            std::cmp::Ordering::Greater => right -= 1, // need smaller: move right down
        }
    }
    None
}

fn main() {
    // Capacity: room already allocated. Pre-size when you know the length,
    // to avoid repeated re-allocation while pushing.
    let mut v = Vec::with_capacity(10);
    println!("len {} capacity {}", v.len(), v.capacity());
    v.extend([5, 3, 8, 3, 1, 8, 8, 2]);
    println!("len {} capacity {}", v.len(), v.capacity());

    println!("contains 8? {}", v.contains(&8));
    println!("max {:?}, min {:?}", v.iter().max(), v.iter().min());

    v.sort(); // in place, stable
    println!("sorted: {v:?}");
    v.dedup(); // removes CONSECUTIVE duplicates, so sort first
    println!("dedup: {v:?}");
    v.retain(|&x| x != 3); // keep only what passes the test
    println!("retain !=3: {v:?}");
    v.reverse();
    println!("reversed: {v:?}");

    // Searching a sorted Vec quickly.
    let sorted = [1, 2, 5, 8];
    println!("binary_search(5) = {:?}", sorted.binary_search(&5));
    println!(
        "binary_search(4) = {:?} (Err = where it would go)",
        sorted.binary_search(&4)
    );
    println!("pair summing to 10: {:?}", pair_with_sum(&sorted, 10));
    println!("pair summing to 4: {:?}", pair_with_sum(&sorted, 4));

    // Sorting by a key, or with your own comparison.
    let mut words = vec!["banana", "fig", "apple", "kiwi"];
    words.sort_by_key(|w| w.len());
    println!("by length: {words:?}");
    words.sort_by(|a, b| b.cmp(a));
    println!("reverse alphabetical: {words:?}");

    // Looking at neighbours and groups (these work on any slice).
    let temps = [18, 21, 19, 25];
    let rises: Vec<_> = temps.windows(2).map(|w| w[1] - w[0]).collect();
    println!("changes: {rises:?}");
    for pair in temps.chunks(2) {
        println!("chunk {pair:?}");
    }

    // Removing: `remove(i)` shifts everything after i (slow on long Vecs),
    // `swap_remove(i)` moves the last element into the gap (fast, reorders).
    let mut q = vec!['a', 'b', 'c', 'd'];
    q.remove(0);
    q.swap_remove(0);
    println!("{q:?}");
}
