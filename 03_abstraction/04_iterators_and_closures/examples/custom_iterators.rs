//! Beyond the Book: implementing Iterator and IntoIterator for your own types.
//! Run: cargo run -p iterators_and_closures --example custom_iterators

// ---- Iterator: write next(), get ~75 adapter methods for free ----

/// Counts from 1 to 5.
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// ---- IntoIterator: make your collection work in a `for` loop ----

struct Playlist {
    songs: Vec<String>,
}

// `for song in playlist` (consumes the playlist, yields String)
impl IntoIterator for Playlist {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}

// `for song in &playlist` (borrows it, yields &String)
impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.songs.iter()
    }
}

impl Playlist {
    /// The usual companion: a plain `iter()` method.
    fn iter(&self) -> std::slice::Iter<'_, String> {
        self.songs.iter()
    }
}

// ---- An iterator that produces values forever ----
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let value = self.current;
        self.current = self.next;
        self.next += value;
        Some(value) // never None: callers must stop it with take(), take_while()...
    }
}

fn main() {
    let all: Vec<u32> = Counter::new().collect();
    println!("Counter: {all:?}");

    // We only wrote next(), but zip, skip, map, filter and sum all work:
    let sum: u32 = Counter::new()
        .zip(Counter::new().skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("the Book's zip/skip/map/filter chain = {sum}"); // 18

    let playlist = Playlist {
        songs: vec!["Kyenkyenhene".into(), "Yaa Amponsah".into()],
    };
    for song in &playlist {
        println!("borrowed: {song}");
    }
    println!("{} songs via iter()", playlist.iter().count());
    for song in playlist {
        println!("owned: {song}");
    }
    // playlist is consumed now.

    let fib: Vec<u64> = Fibonacci {
        current: 0,
        next: 1,
    }
    .take(10)
    .collect();
    println!("first 10 Fibonacci numbers: {fib:?}");
    let below_100: Vec<u64> = Fibonacci {
        current: 0,
        next: 1,
    }
    .take_while(|&n| n < 100)
    .collect();
    println!("below 100: {below_100:?}");

    // std::iter helpers build iterators without a struct:
    let powers: Vec<u32> = std::iter::successors(Some(1u32), |&n| n.checked_mul(10))
        .take(4)
        .collect();
    println!("successors: {powers:?}");
    let mut n = 0;
    let evens: Vec<i32> = std::iter::from_fn(|| {
        n += 2;
        (n <= 8).then_some(n)
    })
    .collect();
    println!("from_fn: {evens:?}");
}
