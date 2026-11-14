//! Book 16.4: the `Send` and `Sync` marker traits, which are how the compiler
//! knows what may cross between threads.
//!
//! - `Send`: ownership of a value can be MOVED to another thread.
//! - `Sync`: a value can be SHARED (`&T`) between threads. `T` is `Sync`
//!   exactly when `&T` is `Send`.
//!
//! Run: cargo run -p fearless_concurrency --example send_and_sync

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// Compile-time checks: these functions only accept types with the badge.
fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

// 1. Auto traits: a struct is Send/Sync automatically when every field is.
struct Player {
    name: String,
    score: u32,
}

// 2. Opting OUT: a PhantomData of a raw pointer makes this type neither
// Send nor Sync, without storing any pointer.
#[allow(dead_code)]
struct ThreadBound {
    _not_send: PhantomData<*const ()>,
}

// 3. Opting IN by hand is `unsafe`: you promise the compiler it's fine.
// Raw pointers are neither Send nor Sync, so this struct isn't either...
struct StaticBytes {
    ptr: *const u8,
    len: usize,
}
// SAFETY: `ptr` only ever points into a `&'static [u8]`, which is never
// freed and never mutated, so any thread may read it at any time.
unsafe impl Send for StaticBytes {}
unsafe impl Sync for StaticBytes {}

impl StaticBytes {
    fn new(bytes: &'static [u8]) -> Self {
        StaticBytes {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
    fn as_slice(&self) -> &[u8] {
        // SAFETY: built from a valid &'static [u8] in `new`.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

fn main() {
    // These all compile, so all of these types are Send and Sync:
    assert_send::<i32>();
    assert_send::<String>();
    assert_send::<Vec<Player>>();
    assert_send::<Arc<Mutex<Player>>>();
    assert_sync::<Arc<Mutex<Player>>>();
    assert_sync::<RwLock<String>>();
    // RefCell and Cell may MOVE to another thread, but not be SHARED:
    assert_send::<RefCell<i32>>();
    assert_send::<Cell<i32>>();
    // assert_sync::<RefCell<i32>>(); // E0277: `RefCell<i32>` cannot be shared between threads safely
    // assert_send::<std::rc::Rc<i32>>(); // E0277: `Rc<i32>` cannot be sent between threads safely
    // assert_send::<std::sync::MutexGuard<'static, i32>>(); // E0277: `std::sync::MutexGuard<'static, i32>` cannot be sent between threads safely
    // (a lock must be unlocked by the thread that locked it)

    println!("| type             | Send | Sync | why |");
    println!("|------------------|------|------|-----|");
    println!("| i32, String, Vec | yes  | yes  | plain owned data |");
    println!("| Rc<T>            | no   | no   | its counter is not atomic |");
    println!("| Arc<T: Send+Sync>| yes  | yes  | atomic counter |");
    println!("| RefCell<T>       | yes  | no   | its borrow flag is not atomic |");
    println!("| Mutex<T: Send>   | yes  | yes  | the lock makes sharing safe |");
    println!("| MutexGuard<T>    | no   | if T | must unlock on the same thread |");
    println!("| *const T, *mut T | no   | no   | the compiler can't check them |");

    // 1. A struct of Send parts can move to a thread with no extra work.
    let player = Player {
        name: "Ama".into(),
        score: 7,
    };
    let player = thread::spawn(move || Player {
        score: player.score + 1,
        ..player
    })
    .join()
    .unwrap();
    println!(
        "\n1. {} (score {}) moved to a thread and back",
        player.name, player.score
    );

    // 2. A ThreadBound value can't be sent:
    // let tb = ThreadBound { _not_send: PhantomData };
    // thread::spawn(move || drop(tb)); // E0277: `*const ()` cannot be sent between threads safely
    println!("2. ThreadBound stays on its own thread (see the commented line)");

    // 3. Our unsafe-impl'd type can be shared by reference across threads.
    let greeting = StaticBytes::new(b"hello from static memory");
    thread::scope(|s| {
        for id in 0..2 {
            let greeting = &greeting; // sharing &StaticBytes needs Sync
            s.spawn(move || {
                let text = String::from_utf8_lossy(greeting.as_slice());
                println!("3. thread {id} reads: {text}");
            });
        }
    });
}
