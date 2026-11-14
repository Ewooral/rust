//! Book 13.2: processing a series of items with iterators.
//! Run: cargo run -p iterators_and_closures --example iterators

#[derive(PartialEq, Debug)]
struct Shoe {
    size: u32,
    style: String,
}

/// A closure that captures `shoe_size` from the function's parameters.
fn shoes_in_size(shoes: Vec<Shoe>, shoe_size: u32) -> Vec<Shoe> {
    shoes.into_iter().filter(|s| s.size == shoe_size).collect()
}

fn main() {
    // Creating an iterator does nothing yet: iterators are LAZY.
    #[allow(clippy::useless_vec)] // the Book uses a Vec here; an array works too
    let v1 = vec![1, 2, 3];
    let v1_iter = v1.iter();
    for val in v1_iter {
        println!("Got: {val}");
    }

    // The whole Iterator trait rests on ONE required method:
    //     fn next(&mut self) -> Option<Self::Item>;
    // `Item` is an associated type: what the iterator produces.
    let mut it = v1.iter(); // next() changes the iterator, so it must be mut
    assert_eq!(it.next(), Some(&1));
    assert_eq!(it.next(), Some(&2));
    assert_eq!(it.next(), Some(&3));
    assert_eq!(it.next(), None);

    // Three ways to iterate a Vec:
    //   iter()       -> &T      (borrow each item)
    //   iter_mut()   -> &mut T  (change each item in place)
    //   into_iter()  -> T       (take ownership; the Vec is consumed)
    let mut v2 = vec![1, 2, 3];
    for x in v2.iter_mut() {
        *x *= 10;
    }
    let owned: Vec<i32> = v2.into_iter().collect(); // v2 is gone after this
    println!("{owned:?}");

    // CONSUMING adapters call next() until the end, using the iterator up.
    let total: i32 = v1.iter().sum();
    println!("sum = {total}");

    // ITERATOR adapters make a new iterator, and still do nothing until
    // consumed. This line alone would warn: unused `Map` that must be used,
    // "iterators are lazy and do nothing unless consumed".
    // v1.iter().map(|x| x + 1);
    let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
    println!("mapped = {v2:?}");

    let shoes = vec![
        Shoe {
            size: 10,
            style: String::from("sneaker"),
        },
        Shoe {
            size: 13,
            style: String::from("sandal"),
        },
        Shoe {
            size: 10,
            style: String::from("boot"),
        },
    ];
    let in_my_size = shoes_in_size(shoes, 10);
    for s in &in_my_size {
        println!("size {}: {}", s.size, s.style);
    }

    // A chain, read top to bottom like a recipe:
    let answer: u32 = (1..=10)
        .filter(|n| n % 2 == 0) // 2, 4, 6, 8, 10
        .map(|n| n * n) // 4, 16, 36, 64, 100
        .take(3) // 4, 16, 36
        .sum(); // 56
    println!("sum of the first three even squares = {answer}");
}
