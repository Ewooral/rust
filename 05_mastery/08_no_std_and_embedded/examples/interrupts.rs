//! Interrupts: sharing data between `main` and an interrupt handler, safely.
//! Run: cargo run -p no_std_and_embedded --example interrupts
//!
//! On a microcontroller, an interrupt (a timer firing, a button pin
//! changing, a byte arriving on the UART) STOPS `main` between two
//! instructions, runs a handler function, then resumes `main`. Any data
//! both of them touch needs protecting, exactly like data shared between
//! threads. The two usual tools:
//!
//!   1. atomics (`core::sync::atomic`) for single numbers and flags;
//!   2. a critical section (interrupts briefly OFF) around anything bigger.
//!      The `critical-section` crate's `Mutex<RefCell<T>>` is the standard
//!      way; below is a tiny version of the same idea.
//!
//! Here the "hardware" is simulated: the loop in `main` calls the handlers
//! between its own steps, single-threaded, like a real one-core chip.

use core::cell::RefCell;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use no_std_and_embedded::ring::Ring;

// ---- 1. Atomics: one number or flag ----------------------------------------------------
static TICKS: AtomicU32 = AtomicU32::new(0);
static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);

// `static mut` would compile only inside `unsafe`, and in edition 2024 even
// taking a reference to one is refused by a deny-by-default lint:
//     static mut COUNT: u32 = 0;
//     let r = unsafe { &COUNT };
//     error: creating a shared reference to mutable static   (no error code)

// ---- 2. A critical section token -------------------------------------------------------------
/// Proof that interrupts are off. Only `interrupt_free` can create one, and
/// it can't escape the closure (the lifetime ties it there).
struct CriticalSection<'cs>(PhantomData<&'cs ()>);

/// On a real chip: save the interrupt state, disable interrupts, run `f`,
/// restore. (cortex-m's `interrupt::free` and `critical_section::with` do
/// this.) In this single-threaded simulation nothing can interrupt us, so
/// there's nothing to switch off.
fn interrupt_free<R>(f: impl FnOnce(&CriticalSection<'_>) -> R) -> R {
    f(&CriticalSection(PhantomData))
}

/// Data that can only be reached while holding a `CriticalSection`.
struct Mutex<T>(T);

impl<T> Mutex<T> {
    const fn new(value: T) -> Mutex<T> {
        Mutex(value)
    }
    fn borrow<'cs>(&'cs self, _proof: &CriticalSection<'cs>) -> &'cs T {
        &self.0
    }
}

// SAFETY: sound on a single-core chip, where holding a CriticalSection
// means nothing else (no interrupt) can run. Not a general-purpose lock!
unsafe impl<T> Sync for Mutex<T> {}

/// The last few ADC samples, filled by an interrupt, read by main.
static SAMPLES: Mutex<RefCell<Ring<u16, 4>>> = Mutex::new(RefCell::new(Ring::new()));

// ---- The interrupt handlers ---------------------------------------------------------------------
// On a board: `#[interrupt] fn TIM2() { ... }` (cortex-m-rt) or an embassy task.
fn timer_interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

fn button_interrupt() {
    BUTTON_PRESSED.store(true, Ordering::Release);
}

fn adc_interrupt(sample: u16) {
    interrupt_free(|cs| {
        SAMPLES.borrow(cs).borrow_mut().push(sample);
    });
}

fn main() {
    // The simulated hardware: what happens before each step of main's loop.
    let script: [&[&str]; 8] = [
        &["timer"],
        &["adc"],
        &["timer", "adc"],
        &[],
        &["timer", "button"],
        &["adc"],
        &["timer", "adc"],
        &["timer"],
    ];
    let mut next_sample = 400;

    for (step, interrupts) in script.iter().enumerate() {
        for &irq in interrupts.iter() {
            match irq {
                "timer" => timer_interrupt(),
                "button" => button_interrupt(),
                _ => {
                    adc_interrupt(next_sample);
                    next_sample += 7;
                }
            }
        }

        // ---- main's own work ----
        // swap(false) reads AND clears the flag in one step, so a press that
        // happens between "read" and "clear" can't be lost.
        let pressed = BUTTON_PRESSED.swap(false, Ordering::Acquire);
        let average = interrupt_free(|cs| SAMPLES.borrow(cs).borrow().average());
        println!(
            "step {step}: ticks={} average={average:?}{}",
            TICKS.load(Ordering::Relaxed),
            if pressed {
                "  <- button was pressed"
            } else {
                ""
            }
        );
    }
}
