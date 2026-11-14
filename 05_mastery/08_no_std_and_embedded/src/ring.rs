//! A fixed-capacity ring buffer: the last `N` items, no heap.
//!
//! On a microcontroller you often can't (or don't want to) allocate, so
//! collections have their size fixed at compile time with a **const
//! generic**: `Ring<u16, 8>` is always room for exactly 8 numbers, stored
//! inline (on the stack, or in a `static`). When it's full, a new item
//! pushes out the oldest one, which is what you want for "the last few
//! sensor readings".
//!
//! The `heapless` crate has a whole family of these (`Vec`, `String`,
//! `Deque`, queues, maps). This one is small enough to read in one go.

/// The last `N` items pushed, oldest first.
///
/// # Examples
///
/// ```
/// use no_std_and_embedded::ring::Ring;
///
/// let mut ring: Ring<u16, 3> = Ring::new();
/// for reading in [10, 20, 30, 40] {
///     ring.push(reading);
/// }
/// assert_eq!(ring.iter().copied().collect::<Vec<_>>(), [20, 30, 40]);
/// assert_eq!(ring.average(), Some(30));
/// ```
#[derive(Debug, Clone)]
pub struct Ring<T, const N: usize> {
    // `Option<T>` so we don't need `T: Default` or `unsafe` MaybeUninit.
    slots: [Option<T>; N],
    // Where the next push goes.
    next: usize,
    len: usize,
}

impl<T, const N: usize> Ring<T, N> {
    /// An empty ring. `const fn`, so it can initialise a `static`.
    pub const fn new() -> Ring<T, N> {
        Ring {
            // An inline `const { }` block repeats a value that isn't Copy.
            slots: [const { None }; N],
            next: 0,
            len: 0,
        }
    }

    /// Adds an item. When full, the oldest item is pushed out and returned.
    pub fn push(&mut self, item: T) -> Option<T> {
        if N == 0 {
            return Some(item);
        }
        let pushed_out = self.slots[self.next].replace(item);
        self.next = (self.next + 1) % N;
        if self.len < N {
            self.len += 1;
        }
        pushed_out
    }

    /// How many items it holds now.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when nothing has been pushed yet.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// True when the next push will push out the oldest item.
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// The fixed capacity, `N`.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// The most recent item.
    pub fn latest(&self) -> Option<&T> {
        let index = (self.next + N.max(1) - 1) % N.max(1);
        self.slots.get(index)?.as_ref()
    }

    /// The items, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = (self.next + N - self.len) % N.max(1);
        (0..self.len).filter_map(move |i| self.slots[(start + i) % N].as_ref())
    }

    /// Empties the ring.
    pub fn clear(&mut self) {
        *self = Ring::new();
    }
}

impl<T, const N: usize> Default for Ring<T, N> {
    fn default() -> Ring<T, N> {
        Ring::new()
    }
}

impl<const N: usize> Ring<u16, N> {
    /// The average of the items, or `None` when empty. Adds up in a `u32`,
    /// so 8 readings of 1000 can't overflow.
    pub fn average(&self) -> Option<u16> {
        if self.is_empty() {
            return None;
        }
        let sum: u32 = self.iter().map(|&v| u32::from(v)).sum();
        // The average of u16 values always fits in a u16.
        u16::try_from(sum / self.len as u32).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items<const N: usize>(ring: &Ring<u16, N>) -> Vec<u16> {
        ring.iter().copied().collect()
    }

    #[test]
    fn fills_up_then_pushes_out_the_oldest() {
        let mut ring: Ring<u16, 3> = Ring::new();
        assert!(ring.is_empty());
        assert_eq!(ring.push(1), None);
        assert_eq!(ring.push(2), None);
        assert_eq!(ring.push(3), None);
        assert!(ring.is_full());
        assert_eq!(ring.push(4), Some(1));
        assert_eq!(items(&ring), [2, 3, 4]);
        assert_eq!(ring.latest(), Some(&4));
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.capacity(), 3);
    }

    #[test]
    fn iterates_oldest_first_before_it_is_full() {
        let mut ring: Ring<u16, 4> = Ring::new();
        ring.push(7);
        ring.push(8);
        assert_eq!(items(&ring), [7, 8]);
        assert_eq!(ring.latest(), Some(&8));
    }

    #[test]
    fn averages_without_overflow() {
        let mut ring: Ring<u16, 8> = Ring::new();
        assert_eq!(ring.average(), None);
        for _ in 0..20 {
            ring.push(u16::MAX);
        }
        assert_eq!(ring.average(), Some(u16::MAX));
        ring.clear();
        ring.push(1);
        ring.push(2);
        assert_eq!(ring.average(), Some(1), "integer division rounds down");
    }

    #[test]
    fn works_for_types_that_are_not_copy() {
        let mut ring: Ring<String, 2> = Ring::new();
        ring.push("a".to_string());
        ring.push("b".to_string());
        assert_eq!(ring.push("c".to_string()).as_deref(), Some("a"));
    }

    #[test]
    fn zero_capacity_keeps_nothing() {
        let mut ring: Ring<u16, 0> = Ring::new();
        assert_eq!(ring.push(5), Some(5));
        assert!(ring.is_empty());
        assert_eq!(ring.latest(), None);
        assert_eq!(ring.iter().count(), 0);
    }

    #[test]
    fn can_live_in_a_static() {
        // `const fn new` is what makes this possible.
        static EMPTY: Ring<u16, 4> = Ring::new();
        assert!(EMPTY.is_empty());
    }
}
