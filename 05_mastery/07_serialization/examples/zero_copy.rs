//! Zero-copy deserialization: structs that BORROW from the input text.
//! Run: cargo run -p serialization --example zero_copy
//!
//! 1. &'de str fields   2. escapes need Cow   3. #[serde(borrow)]
//! 4. counting the allocations saved   5. Deserialize<'de> vs DeserializeOwned

use std::alloc::{GlobalAlloc, Layout, System};
use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Deserialize;

// ---- 1. Borrowed fields --------------------------------------------------------------
// The lifetime 'a says: a Song can't outlive the text it was read from.
// Serde borrows &str and &[u8] fields automatically.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // the fields are only printed with {:?}
struct Song<'a> {
    title: &'a str,
    artist: &'a str,
    seconds: u32,
}

// ---- 3. Cow: borrow when possible, own when necessary ---------------------------------
// Cow fields only borrow if you ask with #[serde(borrow)]. Without it, every
// Cow is Cow::Owned (a fresh String).
#[derive(Debug, Deserialize)]
struct Comment<'a> {
    #[serde(borrow)]
    text: Cow<'a, str>,
    author: Cow<'a, str>, // no #[serde(borrow)]: always Owned
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OwnedSong {
    title: String,
    artist: String,
    seconds: u32,
}

fn main() {
    let input = String::from(r#"{"title": "Kpanlogo", "artist": "Ama", "seconds": 214}"#);
    let song: Song = serde_json::from_str(&input).unwrap();
    // Proof that nothing was copied: the &str points INTO `input`'s buffer.
    let inside = input
        .as_bytes()
        .as_ptr_range()
        .contains(&song.title.as_ptr());
    println!("1. {song:?}\n   title points inside the input text: {inside}\n");

    // The input must outlive the struct. Dropping it first doesn't compile:
    //     let song: Song = { let text = String::from("..."); serde_json::from_str(&text).unwrap() };
    //     E0597: `text` does not live long enough

    // ---- 2. Escapes break borrowing -----------------------------------------------------
    // "Tab\there" in JSON is NOT the bytes T,a,b,\,t,... it means a real tab
    // character. Those bytes don't exist anywhere in the input, so a &str
    // can't point at them:
    let escaped = r#"{"title": "Tab\there", "artist": "Ama", "seconds": 1}"#;
    let err = serde_json::from_str::<Song>(escaped).unwrap_err();
    println!("2. &str with an escape: {err}\n");

    // ---- 3. Cow handles both ----------------------------------------------------------
    for input in [
        r#"{"text": "plain words", "author": "Kofi"}"#,
        r#"{"text": "line one\nline two", "author": "Kofi"}"#,
    ] {
        let comment: Comment = serde_json::from_str(input).unwrap();
        println!(
            "3. text is {:<8} author is {:<8} {:?}",
            kind(&comment.text),
            kind(&comment.author),
            comment.text
        );
    }

    // ---- 4. What it saves ---------------------------------------------------------------
    let many = format!("[{}]", vec![input.as_str(); 1000].join(","));
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let borrowed: Vec<Song> = serde_json::from_str(&many).unwrap();
    let middle = ALLOCATIONS.load(Ordering::Relaxed);
    let owned: Vec<OwnedSong> = serde_json::from_str(&many).unwrap();
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    println!(
        "\n4. 1000 songs: borrowed {} allocations, owned {} allocations",
        middle - before,
        after - middle
    );
    assert_eq!(borrowed.len(), owned.len());

    // ---- 5. Deserialize<'de> vs DeserializeOwned ---------------------------------------
    // `T: Deserialize<'de>` = "T can be read from text that lives for 'de".
    // `T: DeserializeOwned` = "T can be read from text that lives for ANY time",
    // i.e. it borrows nothing. from_reader needs that, because it reads into a
    // buffer that's thrown away as it goes:
    //     let song: Song = serde_json::from_reader(file).unwrap();
    //     error: implementation of `Deserialize` is not general enough  (no error code)
    let from_reader: OwnedSong = serde_json::from_reader(input.as_bytes()).unwrap();
    println!("\n5. from_reader needs an owned type: {from_reader:?}");
}

// clippy suggests &str, but the point here is to see WHICH Cow variant it is.
#[allow(clippy::ptr_arg)]
fn kind(cow: &Cow<str>) -> &'static str {
    match cow {
        Cow::Borrowed(_) => "Borrowed",
        Cow::Owned(_) => "Owned",
    }
}

// ---- A global allocator that counts (to make section 4 measurable) --------------------
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

struct Counting;

// SAFETY: every call is passed straight to the system allocator.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: same contract as our caller's.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: same contract as our caller's.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;
