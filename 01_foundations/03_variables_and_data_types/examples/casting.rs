//! Beyond the Book: converting between number types with `as`, and why
//! `From` and `TryFrom` are usually safer.
//! Run: cargo run -p variables_and_data_types --example casting

fn main() {
    // Widening is always safe: every u8 fits in a u32.
    let small: u8 = 200;
    println!("200u8 as u32 = {}", small as u32);

    // Narrowing with `as` NEVER fails: it silently keeps the low bits.
    let big: i32 = 300;
    println!("300i32 as u8 = {}  (300 - 256)", big as u8);
    let negative: i32 = -1;
    println!("-1i32 as u32 = {}  (all 32 bits set)", negative as u32);

    // Float to integer: drops the fraction, and clamps out-of-range values.
    let price = 3.99_f64;
    println!("3.99 as i32  = {}  (truncated, not rounded)", price as i32);
    println!("3.99.round() as i32 = {}", price.round() as i32);
    let huge = 1e10_f64;
    println!("1e10 as i32  = {}  (clamped to i32::MAX)", huge as i32);
    println!("-1.5 as u8   = {}  (clamped to 0)", -1.5_f64 as u8);
    // (NaN, "not a number", becomes 0. Clippy warns if you write that cast.)

    // char <-> number.
    let letter = 'A';
    println!("'A' as u32   = {}", letter as u32);
    println!("97u8 as char = {}", 97u8 as char);

    // SAFER: `From` exists only where the conversion can never lose data...
    let widened = u32::from(small);
    let as_float = f64::from(big);
    println!("u32::from(200u8) = {widened}, f64::from(300i32) = {as_float}");

    // ...and `TryFrom` returns a Result where it might not fit.
    match u8::try_from(big) {
        Ok(byte) => println!("{big} fits in a u8: {byte}"),
        Err(e) => println!("u8::try_from({big}) failed: {e}"),
    }
    match u8::try_from(42_i32) {
        Ok(byte) => println!("u8::try_from(42) = {byte}"),
        Err(e) => println!("failed: {e}"),
    }
}
