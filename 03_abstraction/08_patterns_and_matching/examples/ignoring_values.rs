//! Book 19.3 (part 3): ignoring values with `_`, `_name` and `..`.
//! Run: cargo run -p patterns_and_matching --example ignoring_values

struct Point3 {
    x: i32,
    y: i32,
    z: i32,
}

/// `_` works in function parameters too: keep the signature (maybe a trait
/// requires it) without using the value, and without a warning.
fn foo(_: i32, y: i32) {
    println!("_ param:   this code only uses the y parameter: {y}");
}

/// The difference between `_` and `_s` matters for ownership: `_s` BINDS,
/// so it moves the String out; `_` doesn't bind at all, so nothing moves.
// Clippy would suggest `s.is_some()` here, which is better style; we want to
// show the `_` pattern itself.
#[allow(clippy::redundant_pattern_matching)]
fn underscore_does_not_move() {
    let s = Some(String::from("Hello!"));
    if let Some(_) = s {
        println!("_ vs _s:   found a string, and `s` was not moved");
    }
    println!("_ vs _s:   still usable: {s:?}");
    // if let Some(_s) = s { ... }
    // println!("{s:?}");
    //   // E0382: borrow of partially moved value: `s`
}

fn main() {
    foo(3, 4);

    // ---------- Nested _: "there must be a Some here, but I don't care what" ----------
    let mut setting_value = Some(5);
    let new_setting_value = Some(10);
    match (setting_value, new_setting_value) {
        (Some(_), Some(_)) => println!("nested _:  can't overwrite an existing customized value"),
        _ => setting_value = new_setting_value,
    }
    println!("nested _:  setting is {setting_value:?}");

    // ---------- _ in several places ----------
    let numbers = (2, 4, 8, 16, 32);
    let (first, _, third, _, fifth) = numbers;
    println!("many _:    some numbers: {first}, {third}, {fifth}");

    // ---------- _name: a REAL variable that just doesn't warn when unused ----------
    let _unused = 5; // no "unused variable" warning

    underscore_does_not_move();

    // ---------- .. ignores "the rest" ----------
    let origin = Point3 { x: 0, y: 0, z: 0 };
    let Point3 { x, .. } = origin; // instead of y: _, z: _
    println!("..:        x is {x}");

    let (first, .., last) = numbers;
    println!("..:        first {first}, last {last}");
    // `..` may appear only once: which number would `second` be here?
    // let (.., second, ..) = numbers;
    //   // error: `..` can only be used once per tuple pattern

    // `..` skipped y and z. Read them here, or the compiler warns that those
    // fields are never read (dead code):
    println!(
        "..:        the origin is ({}, {}, {})",
        origin.x, origin.y, origin.z
    );
}
