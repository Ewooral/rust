//! Book 20.1 (extern functions) and Beyond the Book: FFI, calling C from
//! Rust and Rust from C.
//! Run: cargo run -p advanced_features --example ffi
//!
//! Every Rust program on Windows and Linux already links the C standard
//! library, so we can call its functions with no build script and no C
//! compiler. To link your OWN C code, add a `build.rs` with the `cc` crate,
//! or use `bindgen` to generate the `extern` block from a C header.

use std::ffi::{CStr, CString, c_char, c_int, c_void};

// An `extern` block declares functions that live in another language's code.
// Edition 2024 requires `unsafe extern`: writing the declaration is itself a
// promise that the signatures match the C code exactly.
//     extern "C" { … }  // error: extern blocks must be unsafe
unsafe extern "C" {
    // `safe fn`: we promise this one is safe to call with ANY argument, so
    // callers don't need an unsafe block. True for abs(); it takes a plain int.
    safe fn abs(input: c_int) -> c_int;

    // Unsafe to call: `s` must point to a valid NUL-terminated string.
    fn strlen(s: *const c_char) -> usize;

    // Sorts any array. It calls the comparison function you pass in: that
    // is C code calling RUST code.
    fn qsort(
        base: *mut c_void,
        count: usize,
        size: usize,
        compare: extern "C" fn(*const c_void, *const c_void) -> c_int,
    );
}

// ---- Rust called from C ----------------------------------------------------
// `extern "C" fn` uses the C calling convention, so C can call it.
// It must never let a panic escape into C; since Rust 1.81 a panic that tries
// to unwind out of an `extern "C"` function aborts the process instead.
extern "C" fn compare_descending(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: qsort only passes pointers to elements of the i32 array we gave it.
    let (a, b) = unsafe { (*a.cast::<i32>(), *b.cast::<i32>()) };
    b.cmp(&a) as c_int // reversed: biggest first
}

// To make a function callable BY NAME from a C program, keep its name
// unmangled and build the crate as a library (`crate-type = ["cdylib"]` or
// `["staticlib"]` in Cargo.toml). A C header would then declare:
//     void call_from_c(void);
// `no_mangle` is an UNSAFE attribute in edition 2024 (two functions with the
// same unmangled name anywhere in the program would be undefined behaviour):
//     #[no_mangle]  // error: unsafe attribute used without unsafe
#[unsafe(no_mangle)]
pub extern "C" fn call_from_c() {
    println!("Just called a Rust function from C!");
}

fn main() {
    // C from Rust, the safe way (declared `safe fn`).
    println!("abs(-3) according to C: {}", abs(-3));

    // C from Rust with a raw pointer. CString adds the NUL terminator C expects.
    let greeting = CString::new("hello, C").expect("no NUL bytes inside");
    // SAFETY: `greeting` is a valid NUL-terminated string that outlives the call.
    let length = unsafe { strlen(greeting.as_ptr()) };
    println!("strlen(\"hello, C\") = {length}");

    // A C string literal (c"…") is a &CStr, already NUL-terminated.
    let literal: &CStr = c"literal";
    println!(
        "c\"literal\" is {} bytes plus a NUL",
        literal.to_bytes().len()
    );

    // Rust from C: qsort (C) calls compare_descending (Rust) many times.
    let mut scores = [42, 7, 99, 13, 58];
    // SAFETY: the pointer, count and element size describe `scores` exactly,
    // and compare_descending reads two i32s, which is what qsort passes it.
    unsafe {
        qsort(
            scores.as_mut_ptr().cast(),
            scores.len(),
            size_of::<i32>(),
            compare_descending,
        );
    }
    println!("sorted by C's qsort, compared by Rust: {scores:?}");

    // A C program linking this crate would call it by name; we can too.
    call_from_c();
}
