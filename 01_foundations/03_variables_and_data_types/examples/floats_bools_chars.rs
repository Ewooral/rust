//! The other scalar types: floating-point numbers, booleans and characters.
//! Run: cargo run -p variables_and_data_types --example floats_bools_chars

fn main() {
    // Floats: f64 is the default (as fast as f32 on modern CPUs, and more
    // precise). Both are IEEE-754 binary floats.
    let x = 2.0; // f64
    let y: f32 = 3.0; // f32
    println!("x = {x}, y = {y}");
    println!("5.0 / 3.0 = {}", 5.0 / 3.0);
    println!("7.5 % 2.0 = {}", 7.5 % 2.0);

    // Binary floats can't store 0.1 exactly, so tiny errors appear.
    let sum = 0.1 + 0.2;
    println!("0.1 + 0.2 = {sum}");
    println!("close enough to 0.3? {}", (sum - 0.3_f64).abs() < 1e-9);

    // Integers and floats never mix without a conversion.
    let apples = 3;
    let average_weight = 0.25;
    println!("total weight = {}", apples as f64 * average_weight);

    // Booleans: one byte, `true` or `false`.
    let t = true;
    let is_raining: bool = false;
    println!(
        "t && is_raining = {}, t || is_raining = {}, !t = {}",
        t && is_raining,
        t || is_raining,
        !t
    );

    // Characters: SINGLE quotes, one Unicode scalar value, always 4 bytes.
    let c = 'z';
    let z: char = 'ℤ';
    let heart_eyed_cat = '😻';
    println!("{c} {z} {heart_eyed_cat}");
    println!("a char is {} bytes", std::mem::size_of::<char>());
    println!(
        "inside a string, 'z' takes {} byte and '😻' takes {} bytes",
        c.len_utf8(),
        heart_eyed_cat.len_utf8()
    );
}
