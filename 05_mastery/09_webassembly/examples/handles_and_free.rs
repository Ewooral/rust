//! What JavaScript really holds for an exported struct: a number. And why
//! it must call `.free()`.
//! Run: cargo run -p webassembly --example handles_and_free
//!
//! For `#[wasm_bindgen] pub struct Universe`, wasm-bindgen generates roughly:
//!
//!   Rust side                                  JavaScript side
//!   constructor: Box::new(u), Box::into_raw    class Universe { __wbg_ptr = 1048584 }
//!   method:      &mut *ptr, call it            u.tick() passes this.__wbg_ptr
//!   free:        drop(Box::from_raw(ptr))      u.free() passes it, then sets it to 0
//!
//! JavaScript's garbage collector can't see inside WebAssembly memory, so
//! it doesn't know the number is a pointer. Forget `free()`, and the Rust
//! value lives forever: a memory leak. (wasm-bindgen's `--weak-refs` option
//! can also free objects when JavaScript garbage-collects them, through
//! `FinalizationRegistry`; calling `free()` yourself is still the reliable way.)
//!
//! Below, the same three steps, done by hand.

use std::sync::atomic::{AtomicUsize, Ordering};

static ALIVE: AtomicUsize = AtomicUsize::new(0);

struct Counter {
    name: String,
    count: u32,
}

impl Drop for Counter {
    fn drop(&mut self) {
        ALIVE.fetch_sub(1, Ordering::Relaxed);
        println!("    (Rust) dropped {:?} at count {}", self.name, self.count);
    }
}

/// The constructor JavaScript calls: the value moves to the heap, and only
/// its address goes back.
fn counter_new(name: &str) -> usize {
    ALIVE.fetch_add(1, Ordering::Relaxed);
    let boxed = Box::new(Counter {
        name: name.to_string(),
        count: 0,
    });
    Box::into_raw(boxed) as usize
}

/// A method: turn the number back into a reference.
///
/// # Safety
///
/// `ptr` must come from `counter_new` and not have been freed.
unsafe fn counter_bump(ptr: usize) -> u32 {
    // SAFETY: the caller promises ptr is a live Box<Counter>.
    let counter = unsafe { &mut *(ptr as *mut Counter) };
    counter.count += 1;
    counter.count
}

/// `free()`: rebuild the Box and drop it.
///
/// # Safety
///
/// `ptr` must come from `counter_new`, and must not be used again after.
unsafe fn counter_free(ptr: usize) {
    // SAFETY: the caller promises ptr is a live Box<Counter>, used once here.
    drop(unsafe { Box::from_raw(ptr as *mut Counter) });
}

fn main() {
    println!("JavaScript: const a = new Counter(\"clicks\")");
    let a = counter_new("clicks");
    println!("    JavaScript holds only the number {a:#x}");

    println!("JavaScript: a.bump(); a.bump()");
    // SAFETY: `a` is live.
    let n = unsafe {
        counter_bump(a);
        counter_bump(a)
    };
    println!("    -> {n}");

    println!("JavaScript: const b = new Counter(\"forgotten\")  (never freed)");
    let _b = counter_new("forgotten");

    println!("JavaScript: a.free()");
    // SAFETY: `a` is live, and not used after this.
    unsafe { counter_free(a) };
    // wasm-bindgen's JS sets a.__wbg_ptr = 0 here, so a later a.bump()
    // throws "null pointer passed to rust" instead of touching freed memory.

    println!(
        "\nCounters still alive: {} (\"forgotten\" leaked: JavaScript dropped the number, \
         but nothing told Rust)",
        ALIVE.load(Ordering::Relaxed)
    );
}
