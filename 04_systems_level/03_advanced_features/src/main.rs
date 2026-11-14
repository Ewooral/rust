//! Mini project for Book chapter 20: a typed units-and-vectors toolkit.
//!
//! Run:  cargo run -p advanced_features
//! Test: cargo test -p advanced_features
//!
//! The library (src/lib.rs and its modules) does the work; this tour shows
//! each chapter 20 tool doing something useful:
//!   1. a `vec!`-style macro           -> `trail![(0, 0), (3, 4), …]`
//!   2. a supertrait                   -> `OutlinePrint: Display`
//!   3. an associated type             -> `Segments: Iterator<Item = (Vec2, Vec2)>`
//!   4. newtypes + operator overloading -> `Meters / Seconds = MetersPerSecond`
//!   5. function pointers by name      -> `transform::lookup("rotate90")`
//!   6. returned closures              -> `transform::translate`, `transform::chain`
//!   7. safe code over audited unsafe  -> `Trail::scale_in_parallel`
//!   8. procedural macros              -> `#[derive(HelloMacro)]`, `#[trace]`, `shout!`

use advanced_features::transform::{self, Transform};
use advanced_features::{
    HelloMacro, Meters, MetersPerSecond, Millimeters, OutlinePrint, Seconds, Vec2, shout, trace,
    trail,
};

#[derive(HelloMacro)]
struct Toolkit;

fn main() {
    // 1 + 2. A macro builds the trail; the supertrait frames its Display.
    let mut walk = trail![(0, 0), (3, 4), (3, 10), (0, 10)];
    walk.outline_print();

    // 3. Segments is an iterator whose Item is an ASSOCIATED TYPE.
    for (number, (from, to)) in walk.segments().enumerate() {
        println!(
            "leg {}: {from} -> {to} is {}",
            number + 1,
            (to - from).length()
        );
    }

    // 4. Units are newtypes, so they can't be mixed up by accident.
    let distance: Meters = walk.length();
    let time = Seconds(8.0);
    let speed: MetersPerSecond = distance / time;
    println!("\n{distance} in {time} is {speed:.3}");
    println!("one more stride: {}", distance + Millimeters(750.0));
    println!(
        "at 1.5 m/s that takes {:.1}",
        distance / MetersPerSecond(1.5)
    );
    // let oops = distance + time; // E0277: cannot add `Seconds` to `Meters`

    // 5. Function pointers looked up by name.
    let corner = Vec2::new(3.0, 4.0);
    for name in ["rotate90", "flip_x", "double", "teleport"] {
        let found: Option<Transform> = transform::lookup(name);
        match found {
            Some(f) => println!("{name:>8}: {corner} -> {}", f(corner)),
            None => println!("{name:>8}: no such transform"),
        }
    }

    // 6. Closures returned from functions, chained into one recipe.
    let recipe = transform::chain(vec![
        Box::new(transform::scale(0.5)),
        Box::new(transform::translate(Vec2::new(1.0, 1.0))),
    ]);
    walk.apply_with(&recipe);
    println!("\nhalved and nudged: {walk}");

    // 7. Two threads each scale half of the points. The &mut halves come
    //    from raw::split_at_mut, the crate's one audited `unsafe` block.
    walk.scale_in_parallel(10.0);
    println!("scaled x10 in parallel: {walk}");

    // 8. Procedural macros from our own proc-macro crate (./macros).
    Toolkit::hello_macro();
    println!("{}", shout!("chapter twenty, done!"));
    println!("checked {} points", count_points(walk.points().len()));
}

/// `#[trace]` wraps the body so entering and leaving are printed.
#[trace]
fn count_points(n: usize) -> usize {
    n
}
