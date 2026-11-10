//! Working with arrays and slices.

/// Returns the index of the first element equal to `value`, if any.
///
/// ```
/// use collections::arrays::index_of;
/// assert_eq!(index_of(&[1, 2, 3, 4], 3), Some(2));
/// assert_eq!(index_of(&[1, 2, 3, 4], 8), None);
/// ```
pub fn index_of(items: &[i32], value: i32) -> Option<usize> {
    items.iter().position(|&item| item == value)
}

/// Returns the sum of all elements.
pub fn sum(items: &[i32]) -> i32 {
    items.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_first_match() {
        assert_eq!(index_of(&[1, 2, 3, 3], 3), Some(2));
    }

    #[test]
    fn missing_value_is_none() {
        assert_eq!(index_of(&[1, 2, 3, 4], 5), None);
        assert_eq!(index_of(&[], 1), None);
    }

    #[test]
    fn sums_elements() {
        assert_eq!(sum(&[1, 2, 3, 4]), 10);
        assert_eq!(sum(&[]), 0);
    }
}
