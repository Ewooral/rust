//! What a .wasm file actually is: 41 bytes, decoded and run by hand.
//! Run: cargo run -p webassembly --example wasm_module_bytes
//!
//! WebAssembly is a portable bytecode for a simple STACK MACHINE. A module
//! is a list of sections: types, functions, exports, code... This file holds
//! a complete module, written byte by byte, that exports one function:
//!
//!     (module
//!       (func (export "add") (param i32 i32) (result i32)
//!         local.get 0
//!         local.get 1
//!         i32.add))
//!
//! `rustc --target wasm32-unknown-unknown` produces the same kind of file,
//! just much bigger. Browsers, Node.js and wasmtime run it.

const MODULE: [u8; 41] = [
    0x00, 0x61, 0x73, 0x6D, // magic: "\0asm"
    0x01, 0x00, 0x00, 0x00, // version 1
    0x01, 0x07, // section 1 (type), 7 bytes:
    0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F, //   1 type: func (i32, i32) -> i32
    0x03, 0x02, // section 3 (function), 2 bytes:
    0x01, 0x00, //   1 function, of type 0
    0x07, 0x07, // section 7 (export), 7 bytes:
    0x01, 0x03, b'a', b'd', b'd', 0x00, 0x00, //   1 export: "add" = function 0
    0x0A, 0x09, // section 10 (code), 9 bytes:
    0x01, 0x07, 0x00, //   1 body, 7 bytes, no extra locals
    0x20, 0x00, //   local.get 0
    0x20, 0x01, //   local.get 1
    0x6A, //   i32.add
    0x0B, //   end
];

/// Reads an unsigned LEB128 number: 7 bits per byte, high bit = "more".
/// Every count and size in a wasm file is written this way.
fn leb128(bytes: &[u8], pos: &mut usize) -> u32 {
    let (mut result, mut shift) = (0u32, 0);
    loop {
        let byte = bytes[*pos];
        *pos += 1;
        result |= u32::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            return result;
        }
        shift += 7;
    }
}

fn section_name(id: u8) -> &'static str {
    match id {
        1 => "type",
        2 => "import",
        3 => "function",
        5 => "memory",
        7 => "export",
        10 => "code",
        11 => "data",
        _ => "other",
    }
}

/// A tiny interpreter for the three instructions our function uses.
fn run(code: &[u8], args: &[i32]) -> i32 {
    let mut stack: Vec<i32> = Vec::new();
    let mut pos = 0;
    loop {
        let opcode = code[pos];
        pos += 1;
        match opcode {
            0x20 => {
                let index = leb128(code, &mut pos) as usize;
                stack.push(args[index]);
                println!("    local.get {index}   stack: {stack:?}");
            }
            0x6A => {
                let (b, a) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(a.wrapping_add(b)); // i32.add wraps, like wrapping_add
                println!("    i32.add       stack: {stack:?}");
            }
            0x0B => return stack.pop().unwrap(),
            other => panic!("opcode {other:#04x} isn't in this tiny interpreter"),
        }
    }
}

fn main() {
    assert_eq!(&MODULE[..4], b"\0asm");
    println!(
        "magic {:02X?} (\"\\0asm\"), version {}",
        &MODULE[..4],
        MODULE[4]
    );

    let mut pos = 8;
    let mut body: &[u8] = &[];
    while pos < MODULE.len() {
        let id = MODULE[pos];
        pos += 1;
        let size = leb128(&MODULE, &mut pos) as usize;
        let content = &MODULE[pos..pos + size];
        println!(
            "section {id:>2} ({:<8}) {size} bytes: {content:02X?}",
            section_name(id)
        );
        if id == 7 {
            let name_len = usize::from(content[1]);
            let name = std::str::from_utf8(&content[2..2 + name_len]).unwrap();
            println!("    exports {name:?}");
        }
        if id == 10 {
            // count, body size, local count, then the instructions
            body = &content[3..];
        }
        pos += size;
    }

    println!("\nRunning add(2, 3) on a stack machine:");
    let answer = run(body, &[2, 3]);
    println!("    result: {answer}");
    println!("\nRunning add(i32::MAX, 1): i32 arithmetic wraps, it never panics:");
    println!("    result: {}", run(body, &[i32::MAX, 1]));
}
