//! What does boxing cost? Count heap allocations: a native `async fn` in a
//! trait allocates nothing per call, a boxed future allocates once per call.
//!
//! Run: cargo run -p async_traits_from_scratch --example cost_of_boxing

use std::alloc::{GlobalAlloc, Layout, System};
use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

use async_trait::async_trait;

/// Wraps the normal allocator and counts every allocation.
struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call is passed straight on to `System`, which upholds the
// `GlobalAlloc` contract; we only add a counter.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller's promises about `layout` are passed on unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from `System.alloc` above, with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Runs a future that never has to wait, without any runtime: poll it once.
/// (`Waker::noop()` is a waker that does nothing, fine for a future that
/// is ready at the first poll.)
fn run_ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("this example's futures never wait"),
    }
}

trait NativeService {
    async fn double(&self, n: u64) -> u64;
}

#[async_trait]
trait BoxedService: Send + Sync {
    async fn double(&self, n: u64) -> u64;
}

struct Doubler;

impl NativeService for Doubler {
    async fn double(&self, n: u64) -> u64 {
        n * 2
    }
}

#[async_trait]
impl BoxedService for Doubler {
    async fn double(&self, n: u64) -> u64 {
        n * 2
    }
}

/// Calls `f` 1,000 times and returns how many allocations that took.
fn count_allocations(mut f: impl FnMut(u64) -> u64) -> usize {
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let mut total = 0;
    for n in 0..1_000 {
        total += f(n);
    }
    assert_eq!(total, 999_000); // the sum of 2n for n in 0..1000
    ALLOCATIONS.load(Ordering::Relaxed) - before
}

fn main() {
    let doubler = Doubler;
    let dynamic: &dyn BoxedService = &doubler;

    let native = count_allocations(|n| run_ready(NativeService::double(&doubler, n)));
    let boxed = count_allocations(|n| run_ready(BoxedService::double(&doubler, n)));
    let through_dyn = count_allocations(|n| run_ready(dynamic.double(n)));

    println!("1,000 calls, heap allocations:");
    println!("  native async fn (static dispatch):   {native}");
    println!("  #[async_trait] (static dispatch):    {boxed}");
    println!("  #[async_trait] through &dyn:         {through_dyn}");

    // The boxed future's size is always two pointers (data + vtable); the
    // native one is exactly as big as its state machine.
    let native_future = NativeService::double(&doubler, 1);
    let boxed_future: Pin<Box<dyn Future<Output = u64> + Send + '_>> = dynamic.double(1);
    println!(
        "\nsize of the native future: {} bytes; of the boxed one: {} bytes (+ its heap block)",
        std::mem::size_of_val(&native_future),
        std::mem::size_of_val(&boxed_future),
    );
}
