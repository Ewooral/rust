//! Beyond the Book: const generics and generic associated types (GATs).
//! Run: cargo run -p generics_traits_lifetimes --example const_generics_and_gats

use std::ops::Add;

// ---- Const generics: a VALUE as a generic parameter ----
// The size is part of the type: Matrix<2> and Matrix<3> are different types,
// and mismatched sizes are a compile error, not a runtime check.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Matrix<const N: usize> {
    cells: [[i32; N]; N],
}

impl<const N: usize> Matrix<N> {
    fn identity() -> Self {
        let mut cells = [[0; N]; N];
        for (i, row) in cells.iter_mut().enumerate() {
            row[i] = 1;
        }
        Matrix { cells }
    }

    fn trace(&self) -> i32 {
        (0..N).map(|i| self.cells[i][i]).sum()
    }
}

impl<const N: usize> Add for Matrix<N> {
    type Output = Matrix<N>;
    fn add(self, other: Self) -> Self {
        let mut cells = self.cells;
        for (row, other_row) in cells.iter_mut().zip(other.cells) {
            for (cell, o) in row.iter_mut().zip(other_row) {
                *cell += o;
            }
        }
        Matrix { cells }
    }
}

// Works for arrays of ANY length, with no slice and no runtime length.
fn sum_array<const N: usize>(values: [i32; N]) -> i32 {
    values.iter().sum()
}

// ---- GATs: an associated type that takes its own generic parameter ----
// A "lending iterator" hands out a &mut into ITSELF, so each item must be
// given back before the next is taken. A normal Iterator's `type Item`
// cannot express that; `type Item<'a> where Self: 'a` can.
trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

/// Lends overlapping mutable windows of a slice, one at a time.
struct WindowsMut<'s> {
    slice: &'s mut [i32],
    start: usize,
    size: usize,
}

impl<'s> LendingIterator for WindowsMut<'s> {
    type Item<'a>
        = &'a mut [i32]
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let window = self.slice.get_mut(self.start..self.start + self.size)?;
        self.start += 1;
        Some(window)
    }
}

fn main() {
    let a = Matrix::<3>::identity();
    let b = a + a;
    println!("identity trace {}, doubled trace {}", a.trace(), b.trace());
    // let bad = Matrix::<2>::identity() + Matrix::<3>::identity(); // E0308: expected `Matrix<2>`, found `Matrix<3>`

    println!("sum of [1, 2, 3] = {}", sum_array([1, 2, 3]));
    println!("sum of [5; 10] = {}", sum_array([5; 10]));

    let mut data = [1, 2, 3, 4, 5];
    let mut windows = WindowsMut {
        slice: &mut data,
        start: 0,
        size: 2,
    };
    // Each window doubles its first element. Windows overlap, which a
    // normal iterator of &mut could never allow.
    while let Some(window) = windows.next() {
        window[0] *= 2;
    }
    println!("after lending windows: {data:?}");
}
