//! How a string crosses between JavaScript and WebAssembly (simulated).
//! Run: cargo run -p webassembly --example strings_and_memory
//!
//! A WebAssembly module has ONE block of memory, its "linear memory": a big
//! array of bytes that JavaScript can also see (`memory.buffer`). Functions
//! can only pass NUMBERS (i32, i64, f32, f64). So for `greet(name: &str)`
//! the code wasm-bindgen generates does this:
//!
//!   JavaScript                                   WebAssembly (Rust)
//!   "Ama 👋" is UTF-16 ---- TextEncoder ----->   bytes are UTF-8
//!   ptr = malloc(len); copy bytes to ptr          greet(ptr, len) reads them
//!                                                 writes the answer into memory
//!   TextDecoder <---- (ptr, len) of the answer    returns (ptr, len)
//!   copy out, free(ptr)
//!
//! Every string is copied in and copied out. Here both sides are Rust, and
//! "memory" is a Vec<u8>, so you can watch it happen.

/// The module's linear memory, with the simplest allocator there is: a
/// "bump" pointer that only moves forward.
struct LinearMemory {
    bytes: Vec<u8>,
    next: usize,
}

impl LinearMemory {
    fn malloc(&mut self, len: usize) -> usize {
        let ptr = self.next;
        self.next += len;
        assert!(self.next <= self.bytes.len(), "out of memory");
        ptr
    }
}

/// The Rust function being "called from JavaScript". It only gets numbers.
fn greet(memory: &mut LinearMemory, ptr: usize, len: usize) -> (usize, usize) {
    let name = std::str::from_utf8(&memory.bytes[ptr..ptr + len]).expect("valid UTF-8");
    let answer = format!("Hello, {name}!");
    let out = memory.malloc(answer.len());
    memory.bytes[out..out + answer.len()].copy_from_slice(answer.as_bytes());
    (out, answer.len())
}

fn main() {
    let mut memory = LinearMemory {
        bytes: vec![0; 64],
        next: 8, // address 0 is kept free, like a null pointer
    };

    // ---- JavaScript side, going in ---------------------------------------------------------
    let name = "Ama 👋";
    let utf16: Vec<u16> = name.encode_utf16().collect(); // how JS stores it
    let utf8 = name.as_bytes(); // what TextEncoder makes
    println!(
        "{name:?} in JavaScript: .length = {} (UTF-16 code units)",
        utf16.len()
    );
    println!(
        "{name:?} in Rust:       .len()  = {} (UTF-8 bytes), {} chars",
        utf8.len(),
        name.chars().count()
    );

    let ptr = memory.malloc(utf8.len());
    memory.bytes[ptr..ptr + utf8.len()].copy_from_slice(utf8);
    println!(
        "\ncopied in:  ptr={ptr} len={}  memory[0..32] = {:?}",
        utf8.len(),
        &memory.bytes[..32]
    );

    // ---- The call: only numbers cross ---------------------------------------------------------
    let (out_ptr, out_len) = greet(&mut memory, ptr, utf8.len());
    println!(
        "greet({ptr}, {}) returned ({out_ptr}, {out_len})",
        utf8.len()
    );

    // ---- JavaScript side, coming out -----------------------------------------------------------
    let answer = String::from_utf8(memory.bytes[out_ptr..out_ptr + out_len].to_vec()).unwrap();
    println!("copied out: {answer:?}");
    println!(
        "memory used: {} of {} bytes (a real allocator would free them again)",
        memory.next,
        memory.bytes.len()
    );

    // ---- The lesson -----------------------------------------------------------------------------
    println!("\nNumbers cross for free. Strings and Vecs are COPIED, both ways.");
    println!("For big data (the Game of Life grid), JavaScript reads Rust's memory");
    println!("in place instead: new Uint8Array(memory.buffer, ptr, len). No copy.");
}
