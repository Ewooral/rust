//! Solution to exercise 6: pass a list of structs as one flat buffer (pass 4 tests).
//!
//! wasm-bindgen can't pass a `Vec<Point>` of plain structs to JavaScript.
//! You could export `Point` as a class, but then every point is a separate
//! JavaScript object holding a pointer. For thousands of points, the fast
//! way is ONE flat array of numbers that JavaScript reads as a typed array:
//!
//!     Rust: [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }]
//!     JS:   Float32Array [1, 2, 3, 4]          (x0, y0, x1, y1, ...)
//!     or the raw bytes, little-endian, 8 per point: what `new
//!     Float32Array(bytes.buffer)` reads (WebAssembly is little-endian).
//!
//! Implement:
//!   - `to_floats` / `from_floats`: points <-> [x0, y0, x1, y1, ...].
//!     An odd number of floats is `Err(format!("{n} floats is not a whole number of points"))`.
//!   - `to_bytes` / `from_bytes`: the same, as little-endian bytes
//!     (`f32::to_le_bytes`, `f32::from_le_bytes`). A length that isn't a
//!     multiple of 8 is `Err(format!("{n} bytes is not a whole number of points"))`.
//!
//! `values.as_chunks::<2>()` splits a slice into `[f32; 2]` arrays plus a
//! leftover slice (empty when the length was right); `as_chunks::<4>()` on
//! the bytes gives the `[u8; 4]` that `f32::from_le_bytes` wants.
//!
//! Test: cargo test -p webassembly --features solutions --bin sol06_flat_buffers

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

fn to_floats(points: &[Point]) -> Vec<f32> {
    points.iter().flat_map(|p| [p.x, p.y]).collect()
}

fn from_floats(values: &[f32]) -> Result<Vec<Point>, String> {
    let (pairs, rest) = values.as_chunks::<2>();
    if !rest.is_empty() {
        return Err(format!(
            "{} floats is not a whole number of points",
            values.len()
        ));
    }
    Ok(pairs.iter().map(|&[x, y]| Point { x, y }).collect())
}

fn to_bytes(points: &[Point]) -> Vec<u8> {
    to_floats(points)
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect()
}

fn from_bytes(bytes: &[u8]) -> Result<Vec<Point>, String> {
    if !bytes.len().is_multiple_of(8) {
        return Err(format!(
            "{} bytes is not a whole number of points",
            bytes.len()
        ));
    }
    let (words, _) = bytes.as_chunks::<4>(); // nothing left over: 8 divides the length
    let floats: Vec<f32> = words.iter().map(|&b| f32::from_le_bytes(b)).collect();
    from_floats(&floats)
}

fn main() {
    let route = [Point { x: 1.0, y: 2.0 }, Point { x: 3.5, y: -4.0 }];
    println!("floats: {:?}", to_floats(&route));
    println!("bytes:  {:?}", to_bytes(&route));
    println!("back:   {:?}", from_bytes(&to_bytes(&route)));
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUTE: [Point; 2] = [Point { x: 1.0, y: 2.0 }, Point { x: 3.5, y: -4.0 }];

    #[test]
    fn floats_interleave_x_and_y() {
        assert_eq!(to_floats(&ROUTE), [1.0, 2.0, 3.5, -4.0]);
        assert_eq!(from_floats(&[1.0, 2.0, 3.5, -4.0]), Ok(ROUTE.to_vec()));
        assert_eq!(to_floats(&[]), []);
    }

    #[test]
    fn bytes_are_little_endian_f32s() {
        let bytes = to_bytes(&ROUTE[..1]);
        assert_eq!(bytes, [0, 0, 128, 63, 0, 0, 0, 64], "1.0 then 2.0");
        assert_eq!(to_bytes(&ROUTE).len(), 16);
    }

    #[test]
    fn round_trips() {
        assert_eq!(from_bytes(&to_bytes(&ROUTE)), Ok(ROUTE.to_vec()));
        assert_eq!(from_floats(&to_floats(&ROUTE)), Ok(ROUTE.to_vec()));
    }

    #[test]
    fn broken_lengths_are_errors() {
        assert_eq!(
            from_floats(&[1.0, 2.0, 3.0]),
            Err("3 floats is not a whole number of points".to_string())
        );
        assert_eq!(
            from_bytes(&[0; 12]),
            Err("12 bytes is not a whole number of points".to_string())
        );
    }
}
