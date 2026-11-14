//! Solution to exercise 6: break out of two loops at once with a label.
//!
//! (A `return Some((row, column))` from inside the loops would also work,
//! since leaving the function leaves every loop. The label is the tool when
//! there's more work to do after the loops.)
//!
//! Test: cargo test -p control_flow --features solutions --bin sol06_find_in_grid

fn find(grid: [[u32; 4]; 3], target: u32) -> Option<(usize, usize)> {
    let mut found = None;
    'rows: for (row, cells) in grid.iter().enumerate() {
        for (column, &cell) in cells.iter().enumerate() {
            if cell == target {
                found = Some((row, column));
                break 'rows;
            }
        }
    }
    found
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
