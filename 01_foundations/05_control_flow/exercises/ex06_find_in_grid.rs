//! Exercise 6: break out of two loops at once with a label.
//!
//! Implement `find`: return the (row, column) of the FIRST cell equal to
//! `target`, reading the grid row by row, left to right, or `None` if it
//! isn't there. Use nested `for` loops, a `mut` result that starts as
//! `None`, and `break 'label` to stop both loops when you find it.
//!
//! Test: cargo test -p control_flow --features exercises --bin ex06_find_in_grid

#![allow(unused_variables)]

fn find(grid: [[u32; 4]; 3], target: u32) -> Option<(usize, usize)> {
    todo!("search {grid:?} for {target}")
}

fn main() {
    let grid = [[4, 8, 15, 16], [23, 42, 8, 0], [7, 7, 7, 7]];
    println!("42 is at {:?}", find(grid, 42));
    println!("99 is at {:?}", find(grid, 99));
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID: [[u32; 4]; 3] = [[4, 8, 15, 16], [23, 42, 8, 0], [7, 7, 7, 7]];

    #[test]
    fn finds_a_value() {
        assert_eq!(find(GRID, 42), Some((1, 1)));
        assert_eq!(find(GRID, 4), Some((0, 0)));
        assert_eq!(find(GRID, 0), Some((1, 3)));
    }

    #[test]
    fn finds_the_first_match_only() {
        assert_eq!(find(GRID, 8), Some((0, 1)));
        assert_eq!(find(GRID, 7), Some((2, 0)));
    }

    #[test]
    fn missing_values_give_none() {
        assert_eq!(find(GRID, 99), None);
    }
}
