//! Exercise 4: a fixed-capacity Vec (pass 3 tests).
//!
//! `FixedVec<T, N>` holds up to N items in an array, with no heap: the
//! `no_std` stand-in for `Vec`. Unlike the lesson's `Ring`, it never throws
//! anything away. When it's full, `push` refuses and hands the item back.
//!
//! Implement the `todo!()`s:
//!   - `new`: an array of `T::default()` and a length of 0.
//!   - `push`: store at `len` and grow, or `Err(item)` when full.
//!   - `pop`: the last item, or `None` when empty.
//!   - `as_slice`: only the first `len` items.
//!   - `len`, `is_full`.
//!
//! `T: Copy + Default` keeps it simple: the array always holds valid
//! values. (heapless avoids that bound with `MaybeUninit` and `unsafe`.)
//!
//! Test: cargo test -p no_std_and_embedded --features exercises --bin ex04_fixed_vec

#![no_std]
#![allow(unused_variables, dead_code)]

struct FixedVec<T, const N: usize> {
    items: [T; N],
    len: usize,
}

impl<T: Copy + Default, const N: usize> FixedVec<T, N> {
    fn new() -> FixedVec<T, N> {
        todo!()
    }

    fn push(&mut self, item: T) -> Result<(), T> {
        todo!()
    }

    fn pop(&mut self) -> Option<T> {
        todo!()
    }

    fn as_slice(&self) -> &[T] {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn is_full(&self) -> bool {
        todo!()
    }
}

mod host {
    extern crate std;

    pub fn say(line: core::fmt::Arguments) {
        std::println!("{line}");
    }
}

fn main() {
    let mut jobs: FixedVec<u8, 3> = FixedVec::new();
    for job in 1..=4 {
        host::say(format_args!("push {job}: {:?}", jobs.push(job)));
    }
    let popped = jobs.pop();
    host::say(format_args!(
        "pop -> {popped:?}, {} left: {:?}",
        jobs.len(),
        jobs.as_slice()
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_until_full_then_refuse() {
        let mut v: FixedVec<u16, 3> = FixedVec::new();
        assert_eq!(v.len(), 0);
        assert_eq!(v.push(10), Ok(()));
        assert_eq!(v.push(20), Ok(()));
        assert!(!v.is_full());
        assert_eq!(v.push(30), Ok(()));
        assert!(v.is_full());
        assert_eq!(v.push(40), Err(40), "the item comes back");
        assert_eq!(v.as_slice(), [10, 20, 30]);
    }

    #[test]
    fn pop_takes_from_the_end() {
        let mut v: FixedVec<char, 4> = FixedVec::new();
        assert_eq!(v.pop(), None);
        v.push('a').unwrap();
        v.push('b').unwrap();
        assert_eq!(v.pop(), Some('b'));
        assert_eq!(v.as_slice(), ['a']);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn room_again_after_a_pop() {
        let mut v: FixedVec<u8, 1> = FixedVec::new();
        v.push(1).unwrap();
        assert_eq!(v.push(2), Err(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.push(2), Ok(()));
        assert_eq!(v.as_slice(), [2]);
    }
}
