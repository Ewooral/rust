//! Beyond the Book: memory layout. Size, alignment, `repr(C)`, zero-sized
//! types and niches.
//! Run: cargo run -p advanced_features --example memory_layout
//!
//! Reference: https://doc.rust-lang.org/reference/type-layout.html

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::mem::{align_of, offset_of, size_of};
use std::num::NonZeroU32;

// ---- Size and alignment ------------------------------------------------------------
// Every type has a SIZE (bytes) and an ALIGNMENT (its address must be a
// multiple of this). A struct's size is rounded up to its alignment, and
// fields may need PADDING between them.

// The default layout, repr(Rust), lets the compiler REORDER fields to
// waste less space. Field order in memory is not guaranteed.
#[allow(dead_code)]
struct Rusty {
    a: u8,
    b: u32,
    c: u8,
}

// repr(C) keeps the declared order and C's padding rules, so C code (or a
// file format, or hardware) sees exactly this layout. Needed for FFI structs.
#[allow(dead_code)]
#[repr(C)]
struct LikeC {
    a: u8,  // offset 0, then 3 bytes padding so b is 4-aligned
    b: u32, // offset 4
    c: u8,  // offset 8, then 3 bytes padding so the size is a multiple of 4
}

// repr(C) with the fields sorted largest-first needs less padding.
#[allow(dead_code)]
#[repr(C)]
struct LikeCSorted {
    b: u32,
    a: u8,
    c: u8,
}

// repr(packed): no padding at all. Fields may be misaligned, so taking a
// reference to one is an error (E0793); copy the value out instead.
#[allow(dead_code)]
#[repr(C, packed)]
struct Packed {
    a: u8,
    b: u32,
}

// repr(transparent): exactly the layout of the single field. Lets a newtype
// cross an FFI boundary as if it were the inner type.
#[allow(dead_code)]
#[repr(transparent)]
struct Handle(u32);

// Enums: choose the type of the tag, e.g. to match a C enum or a byte format.
#[allow(dead_code)]
#[repr(u8)]
enum Colour {
    Red = 1,
    Green = 2,
    Blue = 4,
}

// ---- Zero-sized types (ZSTs) --------------------------------------------------------
struct Marker; // a unit struct: 0 bytes
#[allow(dead_code)]
struct Typed<T> {
    id: u32,
    kind: PhantomData<T>, // 0 bytes: a label for the type checker only
}

fn main() {
    println!("== primitive sizes and alignments ==");
    for (name, size, align) in [
        ("u8", size_of::<u8>(), align_of::<u8>()),
        ("u16", size_of::<u16>(), align_of::<u16>()),
        ("u32", size_of::<u32>(), align_of::<u32>()),
        ("u64", size_of::<u64>(), align_of::<u64>()),
        ("char", size_of::<char>(), align_of::<char>()),
        ("bool", size_of::<bool>(), align_of::<bool>()),
        ("usize", size_of::<usize>(), align_of::<usize>()),
    ] {
        println!("{name:>6}: size {size}, align {align}");
    }

    println!("\n== struct layout ==");
    println!(
        "Rusty  {{u8, u32, u8}} repr(Rust): {} bytes (fields reordered)",
        size_of::<Rusty>()
    );
    println!(
        "LikeC  {{u8, u32, u8}} repr(C):    {} bytes (declared order + padding)",
        size_of::<LikeC>()
    );
    println!(
        "  offsets: a={} b={} c={}",
        offset_of!(LikeC, a),
        offset_of!(LikeC, b),
        offset_of!(LikeC, c)
    );
    println!(
        "LikeCSorted {{u32, u8, u8}} repr(C): {} bytes",
        size_of::<LikeCSorted>()
    );
    println!(
        "Packed {{u8, u32}} repr(C, packed): {} bytes, align {}",
        size_of::<Packed>(),
        align_of::<Packed>()
    );
    let p = Packed { a: 1, b: 2 };
    let b = p.b; // copying a packed field is fine
    // let r = &p.b; // E0793: reference to field of packed struct is unaligned
    println!("  read p.b by copy: {b}");
    println!(
        "Handle repr(transparent): {} bytes, same as u32",
        size_of::<Handle>()
    );
    println!(
        "Colour repr(u8): {} byte, Blue = {}",
        size_of::<Colour>(),
        Colour::Blue as u8
    );

    println!("\n== zero-sized types ==");
    println!(
        "() is {} bytes, Marker is {} bytes",
        size_of::<()>(),
        size_of::<Marker>()
    );
    println!(
        "Typed<String> is {} bytes: the PhantomData is free",
        size_of::<Typed<String>>()
    );
    println!("[u64; 0] is {} bytes", size_of::<[u64; 0]>());
    // A HashSet<T> is literally a HashMap<T, ()>: the () values take no space.
    println!(
        "HashSet<u32> is {} bytes, HashMap<u32, ()> is {} bytes",
        size_of::<HashSet<u32>>(),
        size_of::<HashMap<u32, ()>>()
    );
    // A Vec of ZSTs never allocates: it only counts.
    let mut markers = Vec::new();
    for _ in 0..1_000_000 {
        markers.push(Marker);
    }
    println!(
        "a Vec of 1,000,000 Markers: len {}, capacity is usize::MAX: {} (it never allocates)",
        markers.len(),
        markers.capacity() == usize::MAX
    );

    println!("\n== niches: Option for free ==");
    // References and Box are never null, so Option uses null to mean None.
    println!(
        "&u32 {} bytes, Option<&u32> {} bytes",
        size_of::<&u32>(),
        size_of::<Option<&u32>>()
    );
    println!(
        "Box<u32> {} bytes, Option<Box<u32>> {} bytes",
        size_of::<Box<u32>>(),
        size_of::<Option<Box<u32>>>()
    );
    println!(
        "NonZeroU32 {} bytes, Option<NonZeroU32> {} bytes",
        size_of::<NonZeroU32>(),
        size_of::<Option<NonZeroU32>>()
    );
    println!(
        "u32 {} bytes, Option<u32> {} bytes (no spare value, needs a tag)",
        size_of::<u32>(),
        size_of::<Option<u32>>()
    );
    println!(
        "bool {} byte, Option<bool> {} byte (bool only uses 0 and 1)",
        size_of::<bool>(),
        size_of::<Option<bool>>()
    );
}
