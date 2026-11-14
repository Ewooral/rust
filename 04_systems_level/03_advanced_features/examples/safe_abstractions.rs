//! Beyond the Book: soundness, and writing safe abstractions over unsafe
//! code. Also `MaybeUninit`.
//! Run: cargo run -p advanced_features --example safe_abstractions
//!
//! Further reading: the Rustonomicon, https://doc.rust-lang.org/nomicon/
//!
//! UNDEFINED BEHAVIOUR (UB) is anything the language says must never happen:
//! reading uninitialised memory, a dangling or misaligned pointer, two live
//! `&mut` to the same value, a `bool` that is 3, a data race … The compiler
//! assumes UB never happens and optimises on that assumption, so a program
//! with UB can do *anything*, including appearing to work.
//!
//! A safe function is SOUND if no safe caller, passing any arguments at all,
//! can cause UB through it. Unsafe code is where you, not the compiler, check
//! that promise.

use std::mem::MaybeUninit;
use std::rc::Rc;

// ---- Unsound vs sound ------------------------------------------------------------
// UNSOUND: a SAFE function that lets a safe caller cause UB.
// `nth_unsound(&[1, 2, 3], 10)` reads memory past the end of the slice, and
// nothing stops a safe caller writing that. Never write this:
//
//     fn nth_unsound(values: &[i32], n: usize) -> i32 {
//         unsafe { *values.as_ptr().add(n) }
//     }

// SOUND option 1: check, so the unsafe code's precondition always holds.
fn nth_checked(values: &[i32], n: usize) -> Option<i32> {
    if n < values.len() {
        // SAFETY: n < len, so the pointer is inside the slice.
        Some(unsafe { *values.as_ptr().add(n) })
    } else {
        None
    }
}

// SOUND option 2: make the FUNCTION unsafe, and document the promise the
// caller must keep. Now the caller writes `unsafe` and takes responsibility.
/// # Safety
///
/// `n` must be less than `values.len()`.
unsafe fn nth_unchecked(values: &[i32], n: usize) -> i32 {
    // SAFETY: the caller promised n < values.len().
    unsafe { *values.as_ptr().add(n) }
}

// ---- A safe type built on MaybeUninit --------------------------------------------
// `ArrayStack<T, N>`: a stack of up to N items with no heap allocation.
// The array starts UNINITIALISED: `MaybeUninit<T>` is "space for a T that may
// not hold one yet". Reading it as a T before writing is UB.
//
// The INVARIANT that makes every method sound:
//     slots[0..len] are initialised, slots[len..N] are not.
// The fields are PRIVATE, so only the methods below can touch them, and each
// one keeps the invariant true.
pub struct ArrayStack<T, const N: usize> {
    slots: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayStack<T, N> {
    pub fn new() -> Self {
        ArrayStack {
            // No unsafe needed: an array of MaybeUninit may be uninitialised.
            slots: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// Gives the value back if the stack is full.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        self.slots[self.len].write(value); // writing is safe
        self.len += 1; // invariant: slot len-1 is now initialised
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1; // first shrink, so the slot is no longer "ours"
        // SAFETY: the slot was below the old len, so it is initialised, and
        // because len has moved down, nothing will read or drop it again.
        Some(unsafe { self.slots[self.len].assume_init_read() })
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: slots[0..len] are initialised (the invariant), and
        // MaybeUninit<T> has the same layout as T.
        unsafe { std::slice::from_raw_parts(self.slots.as_ptr().cast::<T>(), self.len) }
    }
}

impl<T, const N: usize> Default for ArrayStack<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// MaybeUninit never drops its contents, so WE must drop the live items,
// or every String pushed would leak.
impl<T, const N: usize> Drop for ArrayStack<T, N> {
    fn drop(&mut self) {
        for slot in &mut self.slots[..self.len] {
            // SAFETY: slots below len are initialised, and each is dropped once.
            unsafe { slot.assume_init_drop() };
        }
    }
}

fn main() {
    println!("== sound alternatives to an unchecked read ==");
    let values = [10, 20, 30];
    println!("nth_checked(.., 1)  = {:?}", nth_checked(&values, 1));
    println!("nth_checked(.., 10) = {:?}", nth_checked(&values, 10));
    // SAFETY: 2 < 3.
    println!("nth_unchecked(.., 2) = {}", unsafe {
        nth_unchecked(&values, 2)
    });

    println!("\n== ArrayStack: a safe API over MaybeUninit ==");
    let mut stack: ArrayStack<String, 3> = ArrayStack::new();
    for word in ["safe", "sound", "simple", "spare"] {
        match stack.push(word.to_string()) {
            Ok(()) => println!("pushed {word:?}"),
            Err(back) => println!("full! got {back:?} back"),
        }
    }
    println!("as_slice: {:?}", stack.as_slice());
    println!("pop: {:?}", stack.pop());
    println!("as_slice: {:?}", stack.as_slice());

    // Our Drop impl frees whatever is still inside. Rc counts its owners, so
    // we can watch the items being dropped.
    let tracker = Rc::new(());
    {
        let mut owners: ArrayStack<Rc<()>, 2> = ArrayStack::new();
        owners.push(Rc::clone(&tracker)).unwrap();
        owners.push(Rc::clone(&tracker)).unwrap();
        println!(
            "owners while the stack lives: {}",
            Rc::strong_count(&tracker)
        );
    }
    println!(
        "owners after it is dropped:   {}",
        Rc::strong_count(&tracker)
    );

    println!("\n== MaybeUninit vs the alternatives ==");
    // let x: i32 = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    //     UB: an i32 must be initialised. rustc warns: "the type `i32` does not
    //     permit being left uninitialized … this code causes undefined behavior when executed"
    // std::mem::uninitialized() and zeroed() for such cases are deprecated or dangerous.
    let mut buffer = [MaybeUninit::<u8>::uninit(); 4];
    for (i, slot) in buffer.iter_mut().enumerate() {
        slot.write(i as u8 * 10);
    }
    // SAFETY: every element was written in the loop above.
    let ready: [u8; 4] = buffer.map(|b| unsafe { b.assume_init() });
    println!("filled one slot at a time: {ready:?}");

    println!("\nChecking for UB: run tests under Miri (nightly):");
    println!("  rustup +nightly component add miri");
    println!("  cargo +nightly miri test -p advanced_features");
}
