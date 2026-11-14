//! Which Rust types can cross to JavaScript, and what they become there.
//! Run: cargo run -p webassembly --example passing_data
//!
//! Every function below carries `#[wasm_bindgen]`, and the macro checks each
//! signature even when compiling for the PC: a type it can't pass is a
//! compile error (exercise 1 shows one). So this file compiling IS the check
//! that wasm-bindgen accepts these types.

use wasm_bindgen::prelude::*;

// ---- Numbers and bools: passed as they are, no copying ---------------------------------
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[wasm_bindgen]
pub fn average(a: f64, b: f64) -> f64 {
    (a + b) / 2.0
}

#[wasm_bindgen]
pub fn is_even(n: u32) -> bool {
    n.is_multiple_of(2)
}

// u64 and i64 become JavaScript BigInt (10n), not number.
#[wasm_bindgen]
pub fn big(n: u64) -> u64 {
    n * 2
}

// ---- Option: None becomes undefined --------------------------------------------------------
#[wasm_bindgen]
pub fn first_digit(text: &str) -> Option<u32> {
    text.chars().find_map(|c| c.to_digit(10))
}

// ---- Strings: copied in (UTF-16 -> UTF-8) and copied out ------------------------------------
#[wasm_bindgen]
pub fn shout(text: &str) -> String {
    text.to_uppercase()
}

// ---- Slices and Vecs of numbers: typed arrays (Uint8Array, Float32Array), copied -----------
#[wasm_bindgen]
pub fn sum(values: &[f32]) -> f32 {
    values.iter().sum()
}

#[wasm_bindgen]
pub fn ramp(len: u8) -> Vec<u8> {
    (0..len).collect()
}

// &mut [u8]: copied in, and copied BACK when the function returns.
#[wasm_bindgen]
pub fn double_in_place(values: &mut [u8]) {
    for v in values {
        *v = v.saturating_mul(2);
    }
}

// ---- Vec<String>: an array of strings ------------------------------------------------------
#[wasm_bindgen]
pub fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}

// ---- Errors: Ok returns the value, Err THROWS a JavaScript Error ---------------------------
#[wasm_bindgen]
pub fn parse_percent(text: &str) -> Result<u8, JsError> {
    let n: u8 = text.trim_end_matches('%').parse()?; // ParseIntError -> JsError via `?`
    if n > 100 {
        return Err(JsError::new("a percentage is at most 100"));
    }
    Ok(n)
}

// ---- Structs: a JavaScript class holding a pointer (see handles_and_free) ------------------
#[wasm_bindgen]
pub struct Point {
    pub x: f32, // pub Copy fields become getters/setters: p.x = 3
    pub y: f32,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    pub fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }
}

fn main() {
    println!("Rust type            JavaScript type          example call on the PC");
    println!(
        "i32, u32, f32, f64   number                   add(2, 3) = {}",
        add(2, 3)
    );
    println!(
        "f64                  number                   average(1, 2) = {}",
        average(1.0, 2.0)
    );
    println!(
        "bool                 boolean                  is_even(4) = {}",
        is_even(4)
    );
    println!(
        "u64, i64             BigInt                   big(21) = {}",
        big(21)
    );
    println!(
        "Option<u32>          number | undefined       first_digit(\"ab7\") = {:?}",
        first_digit("ab7")
    );
    println!(
        "&str, String         string (copied)          shout(\"hi\") = {:?}",
        shout("hi")
    );
    println!(
        "&[f32]               Float32Array (copied)    sum([1.5, 2.5]) = {}",
        sum(&[1.5, 2.5])
    );
    println!(
        "Vec<u8>              Uint8Array (copied)      ramp(4) = {:?}",
        ramp(4)
    );
    let mut values = [1, 2, 200];
    double_in_place(&mut values);
    println!(
        "&mut [u8]            Uint8Array, copied back  double_in_place([1, 2, 200]) -> {values:?}"
    );
    println!(
        "Vec<String>          string[]                 words(\"a b\") = {:?}",
        words("a b")
    );
    println!(
        "Result<u8, JsError>  u8, or throws Error      parse_percent(\"42%\") = {:?}",
        parse_percent("42%").ok()
    );
    let p = Point::new(3.0, 4.0);
    println!(
        "#[wasm_bindgen] struct  class instance        new Point(3, 4).length() = {}",
        p.length()
    );
    // (On the PC, building a JsError for the Err case would panic: it needs
    // JavaScript to create the Error object. Test the Ok paths here, and the
    // error logic in plain Rust functions, like the mini project's parser.)
}
