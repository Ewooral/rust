//! The crate's only `unsafe` code: one small block, behind a safe function,
//! with its reasoning written down (Book 20.1, "Creating a Safe Abstraction
//! over Unsafe Code").
//!
//! The rule this module follows: no matter what arguments SAFE code passes,
//! `split_at_mut` must not cause undefined behaviour. That is what makes it
//! *sound*, and why the `assert!` is not optional.

use std::slice;

/// Splits one mutable slice into two non-overlapping mutable slices,
/// `[0, mid)` and `[mid, len)`. The same as std's `<[T]>::split_at_mut`,
/// written out by hand.
///
/// The borrow checker can't do this with indexing, because it sees two
/// `&mut` borrows of the same slice (E0499) and does not know they don't
/// overlap. We do know, so we take responsibility with `unsafe`.
///
/// # Panics
///
/// If `mid > values.len()`.
///
/// ```
/// let mut numbers = [1, 2, 3, 4, 5];
/// let (left, right) = advanced_features::raw::split_at_mut(&mut numbers, 2);
/// left[0] = 10;
/// right[0] = 30;
/// assert_eq!(numbers, [10, 2, 30, 4, 5]);
/// ```
pub fn split_at_mut<T>(values: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();

    // This check is what makes the function sound. Without it, safe code
    // could ask for `mid = 100` and get a slice pointing past the end.
    assert!(
        mid <= len,
        "mid ({mid}) is past the end of the slice ({len})"
    );

    // SAFETY:
    // - `ptr` comes from a valid `&mut [T]` of `len` elements, so it is
    //   non-null, aligned, and points to `len` initialised values.
    // - `mid <= len` (checked above), so `ptr.add(mid)` stays inside the
    //   slice (or points exactly one past its end, which is allowed), and
    //   both new slices lie entirely within the original.
    // - The ranges `[0, mid)` and `[mid, len)` do not overlap, so the two
    //   `&mut` slices never alias each other.
    // - Both returned slices borrow from `values` (same lifetime), so the
    //   original can't be used again until they are gone.
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_are_independent_and_cover_everything() {
        let mut v = vec![1, 2, 3, 4, 5, 6];
        let (a, b) = split_at_mut(&mut v, 4);
        assert_eq!((a.len(), b.len()), (4, 2));
        a[3] = 40;
        b[0] = 50;
        assert_eq!(v, [1, 2, 3, 40, 50, 6]);
    }

    #[test]
    fn edges_are_allowed() {
        let mut v = [1, 2, 3];
        assert_eq!(split_at_mut(&mut v, 0).0.len(), 0);
        assert_eq!(split_at_mut(&mut v, 3).1.len(), 0);
        let mut empty: [u8; 0] = [];
        assert_eq!(split_at_mut(&mut empty, 0), (&mut [][..], &mut [][..]));
    }

    #[test]
    #[should_panic(expected = "past the end")]
    fn mid_past_the_end_panics_instead_of_corrupting_memory() {
        let mut v = [1, 2, 3];
        split_at_mut(&mut v, 4);
    }
}
