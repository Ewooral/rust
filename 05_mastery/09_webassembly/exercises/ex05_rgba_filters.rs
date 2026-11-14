//! Exercise 5: image filters on raw RGBA pixels (pass 3 tests).
//!
//! A browser gives you a picture's pixels with `ctx.getImageData(...)`:
//! a flat array of bytes, four per pixel, row after row:
//!
//!     [R, G, B, A,  R, G, B, A,  ...]
//!
//! Image processing is a classic job for WebAssembly: a tight loop over
//! numbers, no DOM. Pass the bytes in as `&mut [u8]` (wasm-bindgen copies
//! them in and back out; `wasm_bindgen::Clamped` handles the
//! `Uint8ClampedArray` that ImageData really uses), and put them back with
//! `ctx.putImageData`.
//!
//! Implement, leaving alpha (every 4th byte) unchanged:
//!   - `invert`: each colour byte c becomes 255 - c.
//!   - `grayscale`: R, G and B all become (299*R + 587*G + 114*B) / 1000
//!     (do the maths in u32).
//!   - `brighten`: add `amount` (may be negative) to R, G and B, clamping
//!     to 0..=255. (Hint: `i16::from(c).saturating_add(amount).clamp(0, 255)`,
//!     then back to u8; saturating_add can't overflow even for amount = i16::MAX.)
//!
//! `let (whole, _rest) = pixels.as_chunks_mut::<4>();` gives you the pixels
//! as `[u8; 4]` arrays, one at a time (and any leftover bytes in `_rest`).
//!
//! Test: cargo test -p webassembly --features exercises --bin ex05_rgba_filters

#![allow(unused_variables)]

fn invert(pixels: &mut [u8]) {
    todo!()
}

fn grayscale(pixels: &mut [u8]) {
    todo!()
}

fn brighten(pixels: &mut [u8], amount: i16) {
    todo!()
}

fn main() {
    // A 2x1 picture: one orange pixel, one half-transparent blue one.
    let mut pixels = [255, 136, 0, 255, 30, 60, 200, 128];
    println!("original:  {pixels:?}");
    invert(&mut pixels);
    println!("inverted:  {pixels:?}");
    grayscale(&mut pixels);
    println!("grayscale: {pixels:?}");
    brighten(&mut pixels, -50);
    println!("darker:    {pixels:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invert_flips_colours_but_not_alpha() {
        let mut pixels = [255, 136, 0, 255, 30, 60, 200, 128];
        invert(&mut pixels);
        assert_eq!(pixels, [0, 119, 255, 255, 225, 195, 55, 128]);
        invert(&mut pixels);
        assert_eq!(
            pixels,
            [255, 136, 0, 255, 30, 60, 200, 128],
            "twice is a no-op"
        );
    }

    #[test]
    fn grayscale_weights_green_most() {
        let mut pixels = [255, 0, 0, 10, 0, 255, 0, 20, 0, 0, 255, 30];
        grayscale(&mut pixels);
        assert_eq!(pixels, [76, 76, 76, 10, 149, 149, 149, 20, 29, 29, 29, 30]);
    }

    #[test]
    fn brighten_clamps_both_ways() {
        let mut pixels = [250, 100, 3, 200];
        brighten(&mut pixels, 10);
        assert_eq!(pixels, [255, 110, 13, 200]);
        brighten(&mut pixels, -20);
        assert_eq!(pixels, [235, 90, 0, 200]);
        let mut empty: [u8; 0] = [];
        brighten(&mut empty, 5);
    }
}
