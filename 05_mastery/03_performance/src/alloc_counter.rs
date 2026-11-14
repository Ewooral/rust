//! Counting heap allocations: a measuring tape for memory traffic.
//!
//! Every `String`, `Vec` or `HashMap` that grows asks the allocator for
//! memory. This module wraps the system allocator so a program can count
//! those requests. A program opts in with one line:
//!
//! ```text
//! #[global_allocator]
//! static GLOBAL: performance::alloc_counter::CountingAlloc =
//!     performance::alloc_counter::CountingAlloc;
//! ```
//!
//! Two kinds of counter are kept:
//!
//! - **per thread** ([`count_allocations`]): only allocations made by the
//!   calling thread. Tests use this, because `cargo test` runs tests in
//!   parallel and other tests' allocations must not leak into the count.
//! - **process-wide** ([`count_allocations_all_threads`]): every thread,
//!   including rayon's workers. Only meaningful when nothing else is running.
//!
//! Without the `#[global_allocator]` line both counters stay at zero.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

thread_local! {
    // `const` initialisation and no destructor: reading these never
    // allocates, which matters because the allocator itself reads them.
    static THREAD_ALLOCS: Cell<u64> = const { Cell::new(0) };
    static THREAD_BYTES: Cell<u64> = const { Cell::new(0) };
}

static ALL_ALLOCS: AtomicU64 = AtomicU64::new(0);
static ALL_BYTES: AtomicU64 = AtomicU64::new(0);

/// A global allocator that forwards to the system allocator and counts every
/// `alloc`, `alloc_zeroed` and `realloc` call (a `realloc` is a new trip to
/// the allocator, so it counts as one allocation).
pub struct CountingAlloc;

fn record(bytes: usize) {
    // `try_with` instead of `with`: never panic inside the allocator, even
    // while a thread is shutting down.
    let _ = THREAD_ALLOCS.try_with(|c| c.set(c.get() + 1));
    let _ = THREAD_BYTES.try_with(|c| c.set(c.get() + bytes as u64));
    ALL_ALLOCS.fetch_add(1, Ordering::Relaxed);
    ALL_BYTES.fetch_add(bytes as u64, Ordering::Relaxed);
}

// SAFETY: every method forwards to `System` with the caller's arguments
// unchanged, so `System`'s guarantees hold; counting has no effect on memory.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        // SAFETY: same contract as our caller's.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        // SAFETY: same contract as our caller's.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from `System` via one of the methods above.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(new_size);
        // SAFETY: same contract as our caller's.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// How much allocating some code did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AllocStats {
    /// Number of trips to the allocator (`alloc` + `alloc_zeroed` + `realloc`).
    pub allocations: u64,
    /// Total bytes requested by those trips.
    pub bytes: u64,
}

fn thread_snapshot() -> AllocStats {
    AllocStats {
        allocations: THREAD_ALLOCS.with(Cell::get),
        bytes: THREAD_BYTES.with(Cell::get),
    }
}

fn global_snapshot() -> AllocStats {
    AllocStats {
        allocations: ALL_ALLOCS.load(Ordering::Relaxed),
        bytes: ALL_BYTES.load(Ordering::Relaxed),
    }
}

fn difference(before: AllocStats, after: AllocStats) -> AllocStats {
    AllocStats {
        allocations: after.allocations - before.allocations,
        bytes: after.bytes - before.bytes,
    }
}

/// Runs `f` and counts the allocations **this thread** made while it ran.
///
/// The result of `f` is returned too, so it is dropped *after* counting
/// (freeing memory is not counted anyway).
pub fn count_allocations<R>(f: impl FnOnce() -> R) -> (R, AllocStats) {
    let before = thread_snapshot();
    let result = f();
    (result, difference(before, thread_snapshot()))
}

/// Runs `f` and counts the allocations made by **every thread** while it ran.
pub fn count_allocations_all_threads<R>(f: impl FnOnce() -> R) -> (R, AllocStats) {
    let before = global_snapshot();
    let result = f();
    (result, difference(before, global_snapshot()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_one_allocation_for_with_capacity() {
        let (v, stats) = count_allocations(|| Vec::<u64>::with_capacity(100));
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.bytes, 800);
        assert_eq!(v.capacity(), 100);
    }

    #[test]
    fn counts_growth_as_reallocations() {
        let (_, stats) = count_allocations(|| {
            let mut v = Vec::new();
            for i in 0..100u64 {
                v.push(i);
            }
            v
        });
        // 4, 8, 16, 32, 64, 128: one alloc and five reallocs. The growth
        // strategy is an implementation detail, so only check "several".
        assert!(stats.allocations > 1, "{stats:?}");
    }

    #[test]
    fn borrowing_does_not_allocate() {
        let text = String::from("hello world");
        let (words, stats) = count_allocations(|| text.split(' ').count());
        assert_eq!(words, 2);
        assert_eq!(stats.allocations, 0);
    }
}
