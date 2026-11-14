<!-- hand-written lesson: gen_course must not overwrite -->
# `no_std` and Embedded Rust

**Books:** [The Embedded Rust Book](https://docs.rust-embedded.org/book/) · [The Embedonomicon](https://docs.rust-embedded.org/embedonomicon/) (how `no_std` works underneath) · [Discovery](https://docs.rust-embedded.org/discovery/) (hands-on with a board) · [The embassy book](https://embassy.dev/book/)
· Reference: [the `no_std` attribute](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute), [`#[panic_handler]`](https://doc.rust-lang.org/reference/runtime.html#the-panic_handler-attribute), [`#[global_allocator]`](https://doc.rust-lang.org/reference/runtime.html#the-global_allocator-attribute)
· API docs: [`core`](https://doc.rust-lang.org/core/), [`alloc`](https://doc.rust-lang.org/alloc/), [`embedded-hal`](https://docs.rs/embedded-hal), [`embedded-hal-async`](https://docs.rs/embedded-hal-async), [`heapless`](https://docs.rs/heapless)

Run it: `cargo run -p no_std_and_embedded`  ·  Test it: `cargo test -p no_std_and_embedded`

Rust runs on things with no operating system at all: a $1 microcontroller in a thermostat, a bootloader, a kernel, a WebAssembly module (next lesson). There the standard library's `std` isn't available, because `std` is mostly a friendly face on the operating system. This lesson shows what is left (`core`, and optionally `alloc`), and how embedded Rust is built on top of it. After it you'll be able to:

- tell what lives in `core`, `alloc` and `std`, and write a `#![no_std]` library that is still tested on your PC;
- use `alloc` with an allocator, or avoid the heap with fixed-capacity collections;
- explain `#[panic_handler]`, microcontrollers, PACs, HALs, interrupts and the `embedded-hal` traits;
- write a device driver against `embedded-hal`, test it with simulated hardware, and see how async firmware (`embassy`) works.

**What was run here.** `rustup target list --installed` on this machine printed only `x86_64-pc-windows-msvc`. Nothing in this lesson needs another target: every example, test and exercise compiles and runs on the PC. The commands for a real board are shown in their own section and are clearly marked **not run here**.

> 💡 **The big picture: going camping.** `std` is **living at home**: taps, a fridge, a clock on the wall, a postman (the operating system does all that for you). `alloc` is a **campsite with a cool box**: no house, but you can still store as much food as fits (a heap). `core` is a **backpacking trip**: only what's in your rucksack, which is still a lot (the whole Rust language). A **microcontroller** is a tiny island with no house on it, so you bring everything yourself. And `embedded-hal` is a set of **standard plugs**: any torch fits any battery, whichever shop you bought them from.

## 1. `core`, `alloc` and `std`

| Crate | Needs | Has |
|---|---|---|
| `core` | nothing | the language: `Option`, `Result`, iterators, slices, `str`, `char`, numbers, `Cell`, atomics, `Duration`, `fmt`, `Pin`, `Future`, `sort_unstable` |
| `alloc` | a heap (a global allocator) | `Vec`, `String`, `Box`, `Rc`, `Arc`, `BTreeMap`, `format!`, the stable `sort` |
| `std` | an operating system | everything above, re-exported, plus files, threads, `Instant`, networking, `println!`, `HashMap` |

`std` re-exports `core` and `alloc`, so `std::cmp::Ordering` and `core::cmp::Ordering` are the same type. Two surprises: **`HashMap` is std-only** (its default hasher asks the OS for random seeds; use `BTreeMap`, or `heapless`'s maps), and **`Duration` is in core but `Instant` is not** (an instant needs a clock).

The example is itself `#![no_std]`. It brings `alloc` back with `extern crate alloc;`, and `std` only inside `mod host`, to print on your PC:

```text
core:  sorted [498, 505, 512, 530], total 2045, SM-01, None, cell 2, 1.5s
alloc: ["boot", "boots so far: 1"], Rc count 2, task(21) = 42, {"led": 2, "pump": 1}, [(1, 'z'), (2, 'a'), (2, 'b')]
std:   only inside `mod host` in this file
```

Its comments list the real errors from using std items there: `println!` gives "cannot find macro `println` in this scope" (no code), `std::time::Instant` gives E0433, and `use alloc::collections::HashMap` gives E0432.

> 💡 **Analogy: rucksack, cool box, house.** The rucksack (`core`) goes everywhere. The cool box (`alloc`) needs a campsite with ice (a heap). The house (`std`) needs, well, a house: you can't plug in the fridge on a mountain.

▶ `cargo run -p no_std_and_embedded --example core_alloc_std`

## 2. Writing a `no_std` library

```rust
// src/lib.rs
#![cfg_attr(not(test), no_std)]   // no_std for real builds; tests get std

#[cfg(feature = "alloc")]           // heap features are opt-in
extern crate alloc;
```

- `#![no_std]` swaps the std prelude for the core one: `Vec`, `String`, `Box`, `println!` and `format!` disappear, and `std::` paths stop resolving.
- **Tests run on your PC with std**, because the test harness needs threads, printing and unwinding. `cfg_attr(not(test), no_std)` is the usual way to say "no_std, except under `cargo test`". (Writing plain `#![no_std]` and `extern crate std;` inside `#[cfg(test)] mod tests` also works: every exercise in this lesson does that.)
- **Dependencies must be `no_std` too.** Most crates that support it have a `std` feature on by default, so you turn it off:

```toml
[dependencies]
embedded-hal = "1"
serde = { version = "1", default-features = false, features = ["derive"] }
postcard = { version = "1", default-features = false }

[features]
alloc = ["postcard/alloc"]   # Vec-returning functions, only when asked for
```

`cargo tree -e features` shows what each dependency was built with. Here serde got only `derive`, and `serde_core` only `result`, so no `std`:

```text
$ cargo tree -p no_std_and_embedded -e normal,features --prefix none   (filtered)
embedded-hal v1.0.0
postcard v1.1.3
serde v1.0.229
serde feature "derive"
serde_core v1.0.229
serde_core feature "result"
```

The real proof is building for a target that has no std at all, which this machine doesn't have installed (**not run here**):

```bash
rustup target add thumbv7em-none-eabihf
cargo build -p no_std_and_embedded --lib --target thumbv7em-none-eabihf
```

Code copied from a desktop program fails like this (exercise 1). Ignore the `cargo add std` hint: `std` isn't a crate you can add. Write `core::` instead:

```text
error[E0433]: cannot find module or crate `std` in this scope
  --> 05_mastery\08_no_std_and_embedded\exercises\ex01_fix_std_paths.rs:19:9
   |
19 |     use std::cmp::Ordering;
   |         ^^^ use of unresolved module or unlinked crate `std`
   |
   = help: if you wanted to use a crate named `std`, use `cargo add std` to add it to your `Cargo.toml`
```

> 💡 **Analogy: packing for the island.** Before the boat leaves, you check every bag: "does this need a plug socket?" A crate's `std` feature is the hair dryer: lovely at home, useless on the island, so you leave it behind (`default-features = false`). Testing at home first (`cargo test` with std) is fine, as long as you don't pack the hair dryer.

## 3. `alloc` and allocators

Linking `alloc` gives you `Vec` and friends, but something must hand out the memory. On a PC, std does. On a microcontroller **you** register a global allocator, usually over a fixed block of RAM. Leave it out, and the final link step fails. This is the real error from a `no_std` static library (a scratch file compiled with `rustc --crate-type staticlib -C panic=abort`) that uses `Vec` with no allocator:

```text
error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait
```

With the [`embedded-alloc`](https://docs.rs/embedded-alloc) crate it looks like this. It's adapted from that crate's documentation and was **not compiled here** (it needs a board target):

```rust
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// first thing in main: give it 1 KiB of RAM to manage
const HEAP_SIZE: usize = 1024;
static mut HEAP_MEM: [core::mem::MaybeUninit<u8>; HEAP_SIZE] = [core::mem::MaybeUninit::uninit(); HEAP_SIZE];
unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
```

Even with an allocator, a `no_std` crate must ask for `alloc` by name (exercise 2). `Vec` and `String` are E0425 there, `Vec::new()` is E0433, and `format!` has no code at all (macros are looked up before name resolution hands out codes):

```text
error: cannot find macro `format` in this scope
  --> 05_mastery\08_no_std_and_embedded\exercises\ex02_fix_vec_needs_alloc.rs:39:13
   |
39 |             format!("{} readings, highest {max}", self.readings.len())
   |             ^^^^^^

error[E0425]: cannot find type `Vec` in this scope
  --> 05_mastery\08_no_std_and_embedded\exercises\ex02_fix_vec_needs_alloc.rs:23:19
   |
23 |         readings: Vec<u16>,
   |                   ^^^ not found in this scope
```

Many embedded projects avoid the heap entirely: with 64 KiB of RAM, a `Vec` that grows without limit, or a heap that fragments, can crash a device weeks after it was installed. That's why this lesson's `alloc` feature is **off by default**.

> 💡 **Analogy: the cool box needs ice.** Saying "I've got a cool box" (`extern crate alloc`) isn't enough; somebody has to fill it with ice (`#[global_allocator]`). At home the fridge does it for you (std). On the island you bring your own bag of ice, and when it's gone, it's gone.

## 4. Fixed-capacity data structures

Without a heap, the size goes into the **type**, with a const generic. The memory is reserved at compile time, inline, and nothing is ever allocated:

```rust
pub struct Ring<T, const N: usize> {
    slots: [Option<T>; N],
    next: usize,
    len: usize,
}

impl<T, const N: usize> Ring<T, N> {
    pub const fn new() -> Ring<T, N> {             // const fn: it can initialise a `static`
        Ring { slots: [const { None }; N], next: 0, len: 0 }
    }
    pub fn push(&mut self, item: T) -> Option<T> { /* returns what it pushed out */ }
}
```

The price is that **a push can fail**, and you decide what that means: the lesson's `Ring` (in `src/ring.rs`) pushes out the oldest item, which is right for "the last 4 readings". [`heapless`](https://docs.rs/heapless) (`Vec`, `String`, `Deque`, maps, lock-free queues) refuses and hands the item back:

```text
ring: pushed 455, pushed out Some(500), now [480, 470, 455]
heapless::Vec: full, got "blink" back
heapless::String: "moisture=43%" (12 of 24 bytes)
heapless::String: writing more -> Err(Error)
size_of Ring<u16, 3> = 32 bytes, heapless::Vec<u16, 8> = 24 bytes, std Vec<u16> = 24 bytes + heap
```

Formatting works the same way: anything that implements `core::fmt::Write` can take `write!`, so the mini project's `TextBuf<N>` formats a status line into 80 bytes on the stack, no `String` needed.

> 💡 **Analogy: an egg box.** A shopping bag stretches to fit whatever you buy (a heap `Vec`). An egg box has exactly six holes, printed on the lid (`N` in the type). When it's full you either refuse the seventh egg (`heapless::Vec`) or take out the oldest one to make room (`Ring`). You can never be surprised by how much space it takes.

▶ `cargo run -p no_std_and_embedded --example fixed_capacity`

## 5. Panics without an operating system

On a PC a panic prints a message and unwinds or ends the process. On a microcontroller there's no screen and no process, so a `no_std` **program** must say what a panic does, in exactly one function:

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {   // `!`: it never returns
    loop {}                          // or: log `_info`, then reset the chip
}
```

(That snippet compiles as a `no_std` library with `rustc --crate-type rlib -C panic=abort`.) Facts verified with scratch files:

- **Forget it** in a final `no_std` binary or static library: ``error: `#[panic_handler]` function required, but not found`` (no code).
- **Define it in a library that is also tested** with std: std brings its own, and you get ``error[E0152]: found duplicate lang item `panic_impl` ``. Put `#[cfg(not(test))]` on it, or (better) leave it to the application.
- In practice you add one line, `use panic_probe as _;` or `use panic_halt as _;`: crates whose only job is to provide the handler.
- Boards usually set `panic = "abort"` in the profile: there's nobody to unwind to.

Good firmware treats a panic as a bug, not as error handling. The mini project never panics on bad input: a failed sensor read is counted, the pump is switched off, and the loop goes on.

> 💡 **Analogy: the emergency button.** At home, if something goes badly wrong you call for help (std prints the panic). On the island nobody answers, so before you go you must decide what the big red button does: "sit still and wait" (`loop {}`), or "start the day again" (reset the chip). And there can be only one big red button.

## 6. Microcontrollers in three minutes

A **microcontroller** (MCU) is a whole small computer on one chip: a CPU core, some RAM (often 16-512 KiB), flash memory for the program, and **peripherals**: hardware blocks for GPIO pins, timers, I2C, SPI, UART, ADCs, USB. There's no operating system: your program is the only thing running, from power-on, forever.

- **Peripherals are memory-mapped registers**: writing a bit at a fixed address switches a pin on.
- **Targets** name the CPU: `thumbv6m-none-eabi` (Cortex-M0/M0+, such as the RP2040 on a Raspberry Pi Pico), `thumbv7em-none-eabihf` (Cortex-M4F/M7F, such as many STM32 and nRF52 chips), `riscv32imc-unknown-none-elf` (RISC-V, such as the ESP32-C3). `none` means "no operating system".
- **Flashing and debugging** go through a debug probe; [probe-rs](https://probe.rs) is the Rust tool for it.

> 💡 **Analogy: a toy robot's brain.** A PC is a big office with a manager (the OS) who hands out desks and paper. A microcontroller is the little brain inside a toy robot: one worker, one tiny desk, and wires straight to the robot's arms and eyes (the pins). Nobody hands out anything: the worker does it all.

## 7. PACs, HALs and the `embedded-hal` traits

Embedded Rust is built in layers:

| Layer | What it is | Examples |
|---|---|---|
| **PAC** (peripheral access crate) | every register of one chip as Rust types, generated from the maker's description file | `rp2040-pac`, `stm32f4` |
| **HAL** (hardware abstraction layer) | safe drivers for that chip's pins, timers and buses, which **implement the embedded-hal traits** | `rp2040-hal`, `stm32f4xx-hal`, `esp-hal`, `embassy-rp`, `embassy-stm32` |
| **`embedded-hal`** | just the traits, for no chip in particular | `OutputPin`, `InputPin`, `StatefulOutputPin`, `I2c`, `SpiDevice`, `DelayNs` |
| **Drivers and your app** | generic over the traits | a sensor driver, `blink` below |

```rust
fn blink<P: OutputPin, D: DelayNs>(led: &mut P, delay: &mut D, times: u32) -> Result<(), P::Error> {
    for _ in 0..times {
        led.set_high()?;
        delay.delay_ms(250);
        led.set_low()?;
        delay.delay_ms(250);
    }
    Ok(())
}
```

- Every trait has an associated `Error` type (from `ErrorType`), and each error can say its `kind()`, so generic code can react without knowing the HAL. Pins that can't fail use `core::convert::Infallible`.
- Only one method of each trait is usually required: `I2c::transaction`, `DelayNs::delay_ns`. The rest (`write_read`, `delay_ms`, `toggle`) have default bodies.
- The example runs `blink` on two pretend "boards" (a pin that prints, and one that records), plus an active-low button:

```text
Board 2: recording pin, counting delay
  pin history: [High, Low, High, Low, High, Low]
  waited:      1500 ms
```

> 💡 **Analogy: standard plugs.** Every country used to have its own plug shape (each chip's registers, the PAC). The HAL is an adapter for one country. `embedded-hal` is the **standard socket**: a lamp built for it (a driver) works in every country that has the adapter.

▶ `cargo run -p no_std_and_embedded --example hal_traits`

## 8. A driver written against traits, tested on your PC

The mini project's `SoilSensor` drives a pretend I2C chip, the SM-01. Its "datasheet" is at the top of [`src/sensor.rs`](src/sensor.rs):

```rust
impl<I2C: I2c, D: DelayNs> SoilSensor<I2C, D> {
    pub fn read(&mut self) -> Result<Reading, Error<I2C::Error>> {
        self.i2c.write(ADDRESS, &[reg::CONTROL, START]).map_err(Error::Bus)?;
        self.delay.delay_ms(MEASURE_MS);
        let mut data = [0u8; 4];
        self.i2c.write_read(ADDRESS, &[reg::DATA], &mut data).map_err(Error::Bus)?;
        // ... decode big-endian moisture and temperature, check the range
    }
}
```

Three habits of good drivers:

1. **Own the bus**: `SoilSensor::new` takes the `I2C` by value, so nothing else can talk to the chip behind its back, and `release()` gives it back. (To share one bus between several drivers, the `embedded-hal-bus` crate wraps it.)
2. **Wrap, don't hide, the bus error**: `Error::Bus(E)` keeps the HAL's own error. `?` can't do that conversion on its own (exercise 3):

```text
error[E0277]: `?` couldn't convert the error to `Error<<I2C as embedded_hal::i2c::ErrorType>::Error>`
  --> 05_mastery\08_no_std_and_embedded\exercises\ex03_fix_bus_error_conversion.rs:37:66
   |
37 |             .write_read(ADDRESS, &[STATUS_REGISTER], &mut status)?;
   |              ----------------------------------------------------^ the trait `From<<I2C as embedded_hal::i2c::ErrorType>::Error>` is not implemented for `Error<<I2C as embedded_hal::i2c::ErrorType>::Error>`
```

   `.map_err(Error::Bus)?` fixes it (a tuple variant is a function). `impl<E> From<E> for Error<E>` also compiles (checked in a scratch file), and then plain `?` works. Ignore the compiler's `where` clause suggestion.
3. **Test against a simulation**: [`src/sim.rs`](src/sim.rs) implements `I2c` for `SimSoilChip`, which answers like the datasheet says, reading from a shared `World` made of `Cell`s. The simulation is `no_std` too. The example slips a `LoggingBus` between driver and chip to show the bytes on the wire:

```text
SoilSensor::new checks the chip ID:
    0x36: write [00] read [5A]

read(): start a measurement, wait 5 ms, read 4 bytes from 0x02:
    0x36: write [01, 01]
    0x36: write [02] read [01, AF, 08, 66]
  -> Reading { moisture_permille: 431, temp_centi: 2150 }

The sensor's cable comes loose:
    0x36: write [01, 01]  -> NoAcknowledge
  -> Err(Bus(NoAcknowledge))
```

> 💡 **Analogy: a flight simulator for drivers.** A pilot learns on a simulator that behaves exactly like the real plane's controls. Our driver only knows "the I2C controls", so it can't tell the simulator from a real chip, and every crash happens safely on the ground.

▶ `cargo run -p no_std_and_embedded --example i2c_driver`

## 9. Interrupts

An **interrupt** stops `main` between two instructions, runs a handler (a timer fired, a pin changed, a byte arrived), then resumes `main`. Data they share needs the same care as data shared between threads:

- **Atomics** (`core::sync::atomic`) for single numbers and flags: `TICKS.fetch_add(1, Relaxed)`, `BUTTON_PRESSED.swap(false, Acquire)` (read and clear in one step, so no press is lost).
- **A critical section** (interrupts off for a moment) around anything bigger. The standard tool is the `critical-section` crate's `Mutex<RefCell<T>>`, where only code holding a `CriticalSection` token can reach the data. The example builds a tiny version to show the idea.
- **Not `static mut`.** Edition 2024 refuses even a shared reference to one (a deny-by-default lint, no error code; verified in a scratch file):

```text
error: creating a shared reference to mutable static
 --> sm.rs:4:22
  |
4 |     let r = unsafe { &COUNT };
  = note: shared references to mutable statics are dangerous; it's undefined behavior if the static is mutated or if a mutable reference is created for it while the shared reference lives
```

The example simulates the hardware single-threaded (the loop calls the handlers between its own steps), like a one-core chip:

```text
step 4: ticks=3 average=Some(403)  <- button was pressed
step 5: ticks=3 average=Some(407)
```

> 💡 **Analogy: the doorbell.** You're building a Lego tower (main) when the doorbell rings (an interrupt). You put the brick down, answer the door, and come back. If the visitor leaves a note on the tower while you're away, you might knock it over: so a sticky note on the fridge (an atomic) is fine, but for anything bigger you **lock the door for a moment** (a critical section) while you move it.

▶ `cargo run -p no_std_and_embedded --example interrupts`

## 10. Async firmware: embassy

[embassy](https://embassy.dev) runs `async` tasks on microcontrollers. While every task is waiting (for a timer, a DMA transfer, a pin edge), its executor puts the CPU to sleep (`WFI`, wait for interrupt), and the interrupt that ends the wait wakes it again. No threads, no RTOS, one stack, and a sleepy chip that saves battery.

Drivers for it are written against [`embedded-hal-async`](https://docs.rs/embedded-hal-async), the same traits with `async fn`:

```rust
async fn read_moisture<B: I2c, D: DelayNs>(bus: &mut B, delay: &mut D) -> Result<u16, B::Error> {
    bus.write(sensor::ADDRESS, &[sensor::reg::CONTROL, sensor::START]).await?;
    delay.delay_ms(sensor::MEASURE_MS).await;   // other tasks run meanwhile
    let mut data = [0u8; 2];
    bus.write_read(sensor::ADDRESS, &[sensor::reg::DATA], &mut data).await?;
    Ok(u16::from_be_bytes(data))
}
```

The example builds the whole shape on the PC, without embassy: a `Timer` future over a simulated clock, the async I2C and delay traits, two tasks, and a 40-line executor that polls the tasks and then "sleeps" until the next timer (using `core::task::Waker::noop()` and `core::pin::pin!`):

```text
[    0 ms] led on
[    7 ms] moisture 43.1% (the read took 7 ms of waiting)
[  500 ms] led off
[ 1000 ms] led on
[ 1214 ms] moisture 42.2% (the read took 7 ms of waiting)
...
3 simulated seconds: 36 polls, 17 sleeps. The CPU was busy only while polling.
```

A real embassy executor only polls tasks whose `Waker` was called (ours polls both, which is simpler with two tasks). Lesson [05/01](../01_async_traits_from_scratch/README.md) and [04/02](../../04_systems_level/02_async_await/README.md) explain futures, `Pin` and wakers in depth.

> 💡 **Analogy: a babysitter with three sleeping children.** She can't watch three cots at once, so she sets alarms: "check Ama at 2:00, Kofi at 2:30", and **naps in between**. Each alarm wakes her for one job, then she naps again. That's an executor: tasks are the children, timers are the alarms, and the nap is the chip sleeping.

▶ `cargo run -p no_std_and_embedded --example async_executor`

## 11. On a real board (not run here)

**None of this section was run**: this machine has no embedded target installed and no board attached. It's a Raspberry Pi Pico (RP2040) with embassy, and the feature names were checked against embassy-executor 0.10.0 and embassy-rp 0.10.0 with `cargo info`. For anything else, start from the examples folder of your HAL, which always matches its version.

```bash
rustup target add thumbv6m-none-eabi                 # Cortex-M0+
cargo install probe-rs-tools --locked                # the `probe-rs` flasher and debugger
cargo new --bin pico-blink && cd pico-blink
cargo add embassy-executor --features platform-cortex-m,executor-thread,defmt
cargo add embassy-rp --features rp2040,time-driver,critical-section-impl,defmt
cargo add embassy-time --features defmt
cargo add cortex-m-rt defmt defmt-rtt
cargo add panic-probe --features print-defmt
```

`.cargo/config.toml` (build for the chip; `cargo run` flashes through the probe):

```toml
[build]
target = "thumbv6m-none-eabi"

[target.thumbv6m-none-eabi]
runner = "probe-rs run --chip RP2040"
rustflags = ["-C", "link-arg=--nmagic", "-C", "link-arg=-Tlink.x", "-C", "link-arg=-Tdefmt.x"]
```

`memory.x` (where flash and RAM are, read by the linker script):

```text
MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
```

`src/main.rs`:

```rust
#![no_std]
#![no_main]                     // no normal `main`: embassy provides the entry point

use defmt_rtt as _;             // send log messages through the debug probe
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use panic_probe as _;           // the #[panic_handler]

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);   // the Pico's on-board LED
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
```

Then `cargo run --release` builds, flashes and streams the logs. To run the mini project's firmware there, you'd pass `embassy-rp`'s I2C and pin types (which implement the `embedded-hal` traits) to `SoilSensor::new` and `Controller::new`: the library code doesn't change.

> 💡 **Analogy: from the simulator to the real plane.** Everything you practised still applies; only the cockpit is real now. You need the right ticket (the target), a ground crew (probe-rs), and a map of the airport (`memory.x`).

## The mini project: a self-watering plant pot

`src/` is firmware for a pot that waters its plant: a `no_std` library, and a PC binary that runs it against simulated hardware.

```text
$ cargo run -p no_std_and_embedded
Self-watering pot, 72 hours. Water below 30.0%, stop above 45.0%.
A loose cable unplugs the sensor at 05:00-06:59 on day 3.

time        moisture                      temp     pump
day 1 00:00  ##########..........  52.0%  18.00C  off
...
day 2 04:00  #####...............  28.6%  18.00C  ON   <- dry: pump on
day 2 06:00  ########............  41.2%  19.60C  ON
day 2 07:00  #########...........  47.4%  20.40C  off  <- wet enough: pump off, soaking
day 2 10:00  ########............  44.8%  22.80C  off  <- soaked in: watching again
...
day 3 04:00  #####...............  28.9%  18.00C  ON   <- dry: pump on
day 3 05:00  #####...............  28.9%  18.00C  off  <- SENSOR FAULT: pump off
day 3 06:00  #####...............  28.9%  18.00C  off  <- sensor read failed (last good value shown)
...
Debug console (formatted into a TextBuf<80>, no String):
  t=72h moisture=39.6% temp=18.80C pump=off waterings=3 faults=2
Radio packet (postcard into a [u8; 18] on the stack): 8 bytes [72, 140, 3, 176, 29, 0, 3, 2]
Decoded at the base station: Status { uptime_hours: 72, moisture_permille: 396, temp_centi: 1880, pump_on: false, waterings: 3, sensor_faults: 2 }
```

| File | Lesson topic |
|---|---|
| [`src/lib.rs`](src/lib.rs) | `#![cfg_attr(not(test), no_std)]`, `alloc` behind a feature, `run_hour` (one pass of the firmware loop) |
| [`src/ring.rs`](src/ring.rs) | a fixed-capacity ring buffer with const generics and a `const fn new` |
| [`src/sensor.rs`](src/sensor.rs) | an I2C driver over `embedded_hal::i2c::I2c` + `DelayNs`, owning its bus |
| [`src/controller.rs`](src/controller.rs) | the pump logic over an `OutputPin`: smoothing, hysteresis, a safety cut-off, soak time, fail-safe |
| [`src/report.rs`](src/report.rs) | Serde + postcard into a stack buffer; `encode_vec` with `--features alloc`; `TextBuf` and `core::fmt::Write` |
| [`src/sim.rs`](src/sim.rs) | simulated chip, pump and delay (`no_std`, sharing a `World` of `Cell`s) |
| [`src/main.rs`](src/main.rs) | the PC demo (std): three days of simulated weather |

> 💡 **Analogy: a gardener in a box.** The little brain checks the soil every hour. If the soil has been dry for a while, it turns on the tap; when the soil is wet, it turns it off and waits for the water to soak in. If it can't feel the soil any more (the cable came loose), it turns the tap off, just in case, rather than flooding the kitchen.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `use std::...` in a `no_std` crate | E0433 cannot find module or crate `std` | `core::...` (or `alloc::...`); ignore the `cargo add std` hint. |
| `Vec`, `String`, `Box` in a `no_std` crate | E0425 cannot find type (E0433 for `Vec::new`) | `extern crate alloc;` + `use alloc::vec::Vec;`, or a fixed-capacity type. |
| `format!` / `println!` in `no_std` | cannot find macro (no code) | `use alloc::format;`; for output, `write!` into a buffer or a UART (`defmt` on boards). |
| `use alloc::collections::HashMap` | E0432 unresolved import | `alloc::collections::BTreeMap`, or `heapless`'s maps. |
| `alloc` with no allocator in the final binary | no global memory allocator found (no code) | `#[global_allocator]`, e.g. `embedded-alloc`. |
| No `#[panic_handler]` in a `no_std` binary | `#[panic_handler]` function required (no code) | `use panic_halt as _;` or `use panic_probe as _;`. |
| A `#[panic_handler]` in a library you also test | E0152 duplicate lang item `panic_impl` | Leave it to the binary, or `#[cfg(not(test))]`. |
| A dependency with its default `std` feature | builds on the PC, fails on the board | `default-features = false`; check with `cargo tree -e features`. |
| `?` on a bus error into your own `Error<E>` | E0277 `?` couldn't convert the error | `.map_err(Error::Bus)?`, or `impl<E> From<E> for Error<E>`. |
| `&COUNTER` where `static mut COUNTER` | creating a shared reference to mutable static (deny lint, no code) | An atomic, or a critical-section `Mutex<RefCell<T>>`. |
| Reading and clearing a flag in two steps | a press that arrives in between is lost | `flag.swap(false, Ordering::Acquire)`. |
| One threshold for on and off | the pump chatters on and off | Hysteresis: two thresholds. |
| `unwrap()` on a sensor read in firmware | the device locks up in the field | Handle the error: count it, fail safe, keep looping. |
| `heapless::Vec::push(x)` result ignored | data silently dropped | Handle `Err(x)`: it gives the item back. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p no_std_and_embedded --example core_alloc_std
cargo run -p no_std_and_embedded --example fixed_capacity
cargo run -p no_std_and_embedded --example hal_traits
cargo run -p no_std_and_embedded --example i2c_driver
cargo run -p no_std_and_embedded --example interrupts
cargo run -p no_std_and_embedded --example async_executor
```

**2. Do the exercises** in `exercises/`. Every exercise file is `#![no_std]`; only `mod host` (for printing) and the tests bring std back:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_std_paths` | Make it compile (E0433): `std::` paths in `no_std` code | `cargo run -p no_std_and_embedded --features exercises --bin ex01_fix_std_paths` |
| 2 | `ex02_fix_vec_needs_alloc` | Make it compile (E0425): link and import `alloc` | `cargo run -p no_std_and_embedded --features exercises --bin ex02_fix_vec_needs_alloc` |
| 3 | `ex03_fix_bus_error_conversion` | Make it compile (E0277): wrap the bus error | `cargo run -p no_std_and_embedded --features exercises --bin ex03_fix_bus_error_conversion` |
| 4 | `ex04_fixed_vec` | A fixed-capacity `Vec`, pass 3 tests | `cargo test -p no_std_and_embedded --features exercises --bin ex04_fixed_vec` |
| 5 | `ex05_debounce_button` | Debounce an `InputPin`, pass 3 tests | `cargo test -p no_std_and_embedded --features exercises --bin ex05_debounce_button` |
| 6 | `ex06_light_sensor_driver` | An I2C driver from a datasheet, pass 4 tests | `cargo test -p no_std_and_embedded --features exercises --bin ex06_light_sensor_driver` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p no_std_and_embedded --features solutions --bins`, or `cargo run -p no_std_and_embedded --features solutions --bin sol05_debounce_button` and so on).

**4. The mini project:**
- `cargo run -p no_std_and_embedded -- 240` simulates 10 days; `--features alloc` also shows `encode_vec`.
- `cargo test -p no_std_and_embedded`: 27 unit tests and 2 doc tests (28 unit tests with `--features alloc`).
- Change `Config::HOUSE_PLANT` to a cactus (water rarely, a little). What do the tests say?
- If you have a board: section 11, then pass your HAL's I2C and pin to `SoilSensor::new` and `Controller::new`.

## Checklist

- [ ] `core` vs `alloc` vs `std`: what each needs, what each has
- [ ] Writing a `no_std` library: `#![cfg_attr(not(test), no_std)]`, `default-features = false`, `cargo tree -e features`
- [ ] `alloc` with a global allocator, and why many devices avoid the heap
- [ ] Fixed-capacity data structures: const generics, `Ring`, `heapless`, `core::fmt::Write`
- [ ] Panics in `no_std`: `#[panic_handler]`, `panic-probe`/`panic-halt`, `panic = "abort"`
- [ ] Microcontrollers, peripherals and targets
- [ ] PACs, HALs and the `embedded-hal` traits
- [ ] Drivers over traits, tested with simulated hardware
- [ ] Interrupts: atomics, critical sections, no `static mut`
- [ ] Async firmware: executors, timers, `embedded-hal-async`, embassy
- [ ] What it takes to run on a real board (target, probe-rs, `memory.x`)
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `core` is the language and needs nothing; `alloc` adds heap types and needs an allocator; `std` adds the operating system. A `no_std` crate picks the first one or two.
- A `no_std` library is still tested on your PC (`cfg_attr(not(test), no_std)`), and its dependencies need `default-features = false`.
- Without a heap, sizes live in types (`Ring<T, N>`, `heapless`), and a full buffer is a decision, not a crash. Panics need a handler, and firmware avoids them.
- Chips expose registers (PAC), HALs make them safe and implement `embedded-hal`, and drivers are generic over those traits, so they can be tested against simulations.
- Interrupts share data through atomics and critical sections; embassy runs async tasks and sleeps the chip in between.

**In one sentence for a child:** a tiny computer inside a toy has no grown-up to help it, so we give it only the tools it can carry, standard plugs for its arms and eyes, and a practice robot on the PC to learn on before it goes into the real toy.

**Next:** [09 WebAssembly](../09_webassembly/README.md)
