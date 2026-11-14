//! Book 20.1: the five things `unsafe` lets you do.
//! Run: cargo run -p advanced_features --example unsafe_superpowers
//!
//! `unsafe` does NOT turn off the borrow checker or any other check. It
//! unlocks exactly five extra abilities, and you promise to use them
//! correctly:
//!   1. dereference a raw pointer
//!   2. call an unsafe function or method
//!   3. access or modify a mutable static variable
//!   4. implement an unsafe trait
//!   5. access the fields of a union
//!
//! (Calling C, the sixth thing people mean by "unsafe Rust", is in the `ffi` example.)

use std::slice;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- 1. Raw pointers --------------------------------------------------------
fn raw_pointers() {
    let mut num = 5;

    // Creating raw pointers is SAFE. `&raw const` / `&raw mut` make them
    // directly, without first making a reference.
    let r1 = &raw const num; // *const i32
    let r2 = &raw mut num; // *mut i32: both point at `num`, which references can't do

    // A raw pointer can hold ANY address. Making one is harmless; using it isn't.
    let address = 0x012345usize;
    let _anywhere = address as *const i32; // never dereference this!

    // Dereferencing is the unsafe part.
    // println!("{}", *r1); // E0133: dereference of raw pointer is unsafe and requires unsafe block
    unsafe {
        println!("r1 is: {}", *r1);
        *r2 += 1;
        println!("r2 is: {}", *r2);
    }
    println!("num is now {num}");
}

// ---- 2. Unsafe functions ----------------------------------------------------
/// # Safety
///
/// `index` must be less than `values.len()`.
unsafe fn get_unchecked(values: &[i32], index: usize) -> i32 {
    // Since edition 2024 the body of an `unsafe fn` is NOT one big unsafe
    // block: each unsafe operation still needs its own block (and a reason).
    // SAFETY: the caller promised `index < values.len()`.
    unsafe { *values.as_ptr().add(index) }
}

// A SAFE function wrapping unsafe code (the Book's split_at_mut).
// The naive version fails: E0499 cannot borrow `*values` as mutable more than once
//     (&mut values[..mid], &mut values[mid..])
fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();
    assert!(mid <= len);
    // SAFETY: mid <= len, so both slices are inside `values` and don't overlap.
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

fn unsafe_functions() {
    let numbers = [10, 20, 30];
    // get_unchecked(&numbers, 1); // E0133: call to unsafe function `get_unchecked` is unsafe and requires unsafe block
    // SAFETY: 1 < 3.
    let second = unsafe { get_unchecked(&numbers, 1) };
    println!("second number: {second}");

    let mut v = vec![1, 2, 3, 4, 5, 6];
    let (a, b) = split_at_mut(&mut v, 3);
    a[0] = 100;
    b[0] = 400;
    println!("after split_at_mut: {v:?}");
}

// ---- 3. Mutable statics -----------------------------------------------------
static HELLO_WORLD: &str = "Hello, world!"; // immutable static: always safe to read
static mut COUNTER: u32 = 0; // mutable static: every access is unsafe

/// # Safety
///
/// Calling this from more than one thread at a time is a data race.
unsafe fn add_to_count(inc: u32) {
    // SAFETY: only called from the main thread (see the caller).
    unsafe {
        COUNTER += inc;
    }
}

// The safe alternative: an atomic. No unsafe at all, and thread-safe.
static ATOMIC_COUNTER: AtomicU32 = AtomicU32::new(0);

fn mutable_statics() {
    println!("{HELLO_WORLD}");
    // SAFETY: single-threaded here.
    unsafe { add_to_count(3) };

    // In edition 2024 even a SHARED reference to a static mut is an error:
    // unsafe { println!("{}", COUNTER) }; // error: creating a shared reference to mutable static
    // (println! borrows its arguments, so even printing makes a reference.)
    // Copy the value out (no reference), or use a raw pointer from &raw const:
    // SAFETY: no other thread is writing COUNTER.
    let count = unsafe { COUNTER };
    let ptr = &raw const COUNTER; // making the pointer is safe
    // SAFETY: as above; the pointer is valid and aligned.
    let again = unsafe { ptr.read() };
    println!("COUNTER: {count} (through a raw pointer: {again})");

    ATOMIC_COUNTER.fetch_add(3, Ordering::Relaxed);
    println!("ATOMIC_COUNTER: {}", ATOMIC_COUNTER.load(Ordering::Relaxed));
}

// ---- 4. Unsafe traits -------------------------------------------------------
/// # Safety
///
/// Implementors promise that `zeroed_is_valid` is true only if a value made
/// of all-zero bytes is a valid value of the type. Code may rely on it.
unsafe trait Zeroable {
    fn zeroed_is_valid() -> bool {
        true
    }
}

// SAFETY: every bit pattern, including all zeros, is a valid u64.
unsafe impl Zeroable for u64 {}

// The best-known unsafe traits are Send and Sync. The compiler implements
// them automatically; writing `unsafe impl Send for MyType {}` yourself is a
// promise that sending MyType to another thread really is safe.
struct PointerHolder(*mut u8); // raw pointers are not Send by default
// SAFETY: we never share the pointer; the thread that receives it owns it.
unsafe impl Send for PointerHolder {}

impl PointerHolder {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

fn unsafe_traits() {
    println!("u64 zeroed is valid: {}", u64::zeroed_is_valid());
    let holder = PointerHolder(std::ptr::null_mut());
    // Call a method rather than touching `holder.0` in the closure: a closure
    // captures just the FIELDS it uses, and the bare `*mut u8` is not Send.
    let handle = std::thread::spawn(move || holder.is_null());
    println!(
        "moved a raw pointer to a thread: null = {}",
        handle.join().unwrap()
    );
}

// ---- 5. Unions --------------------------------------------------------------
// A union's fields share the same bytes. Rust doesn't know which field was
// written last, so reading any field is unsafe. (Mostly used for C interop.)
#[repr(C)]
union IntOrFloat {
    i: u32,
    f: f32,
}

fn unions() {
    let mut value = IntOrFloat { f: 1.0 };
    // SAFETY: every bit pattern is a valid u32, so reading `i` is fine.
    let bits = unsafe { value.i };
    println!("1.0_f32 as bits: {bits:#010x}");
    value.i = 0x4049_0fdb; // writing a field is safe
    // SAFETY: these bits are a valid f32 (roughly pi).
    println!("bits 0x40490fdb as f32: {}", unsafe { value.f });
}

fn main() {
    println!("== 1. raw pointers ==");
    raw_pointers();
    println!("\n== 2. unsafe functions ==");
    unsafe_functions();
    println!("\n== 3. mutable statics ==");
    mutable_statics();
    println!("\n== 4. unsafe traits ==");
    unsafe_traits();
    println!("\n== 5. unions ==");
    unions();
}
