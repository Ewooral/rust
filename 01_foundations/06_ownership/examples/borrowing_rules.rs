//! The borrowing rules, and the compile errors that enforce them.
//! Run: cargo run -p ownership --example borrowing_rules
//!
//! At any given time you can have EITHER one mutable reference
//! OR any number of immutable references. References must always be valid.

fn main() {
    let mut v = vec![1, 2, 3];

    // E0499: two mutable borrows alive at once. Uncomment to see it.
    // let a = &mut v;
    // let b = &mut v;
    // a.push(4);
    // b.push(5);

    // E0502: a shared borrow is still in use when we try to mutate.
    // `push` may reallocate the buffer and leave `first` dangling.
    // let first = &v[0];
    // v.push(4);
    // println!("{first}");

    // Fix: copy the i32 out instead of keeping a borrow alive.
    let first = v[0];
    v.push(4);
    println!("first = {first}, v = {v:?}");

    // Mutable borrows that do not overlap are fine.
    {
        let a = &mut v;
        a.push(5);
    }
    let b = &mut v;
    b.push(6);
    println!("{v:?}");
}
