//! Async firmware, embassy-style, with a 40-line executor you can read.
//! Run: cargo run -p no_std_and_embedded --example async_executor
//!
//! embassy (https://embassy.dev) runs async tasks on a microcontroller:
//!
//!     #[embassy_executor::task]
//!     async fn blink(mut led: Output<'static>) {
//!         loop { led.toggle(); Timer::after_millis(500).await; }
//!     }
//!
//! While every task is waiting (on a timer, a DMA transfer, a pin edge), the
//! executor puts the CPU to sleep (WFI: "wait for interrupt"), and the
//! interrupt that ends the wait wakes it again. No threads, no RTOS, one
//! stack, and the chip sleeps most of the time.
//!
//! This file builds the same shape on the PC, without embassy:
//!   - `Timer`: a future that completes when a simulated clock reaches a time;
//!   - `AsyncBus`, `AsyncDelay`: the embedded-hal-ASYNC traits, which embassy's
//!     HALs implement;
//!   - `read_moisture`: an async driver function, generic over those traits;
//!   - `run`: the executor: poll the tasks, then "sleep" to the next timer.

use core::cell::Cell;
use core::future::Future;
use core::pin::{Pin, pin};
use core::task::{Context, Poll, Waker};

use embedded_hal::i2c::{ErrorType, Operation};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use no_std_and_embedded::sensor;
use no_std_and_embedded::sim::{SimI2cError, SimSoilChip, World};

// ---- The clock and its timer future -------------------------------------------------------
/// Simulated time, plus the earliest moment any task asked to be woken.
struct Clock {
    now_ms: Cell<u64>,
    next_wake: Cell<Option<u64>>,
}

impl Clock {
    fn now(&self) -> u64 {
        self.now_ms.get()
    }

    fn after(&self, ms: u64) -> Timer<'_> {
        Timer {
            clock: self,
            at: self.now() + ms,
        }
    }
}

/// Completes when the clock reaches `at`. On a chip, the "wake me" part
/// would program a hardware timer to fire an interrupt at that moment.
struct Timer<'c> {
    clock: &'c Clock,
    at: u64,
}

impl Future for Timer<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.clock.now() >= self.at {
            return Poll::Ready(());
        }
        let earliest = self
            .clock
            .next_wake
            .get()
            .map_or(self.at, |t| t.min(self.at));
        self.clock.next_wake.set(Some(earliest));
        Poll::Pending
    }
}

// ---- The async HAL traits, implemented for the simulation -----------------------------------
struct AsyncDelay<'c> {
    clock: &'c Clock,
}

impl DelayNs for AsyncDelay<'_> {
    async fn delay_ns(&mut self, ns: u32) {
        self.clock.after(u64::from(ns).div_ceil(1_000_000)).await;
    }
}

/// An I2C bus where each transfer takes 1 ms. On a chip the CPU would hand
/// the bytes to DMA and sleep until the "transfer done" interrupt.
struct AsyncBus<'w, 'c> {
    chip: SimSoilChip<'w>,
    clock: &'c Clock,
}

impl ErrorType for AsyncBus<'_, '_> {
    type Error = SimI2cError;
}

impl I2c for AsyncBus<'_, '_> {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), SimI2cError> {
        self.clock.after(1).await;
        embedded_hal::i2c::I2c::transaction(&mut self.chip, address, operations)
    }
}

// ---- An async driver: the same steps as the blocking SoilSensor::read ------------------------
async fn read_moisture<B: I2c, D: DelayNs>(bus: &mut B, delay: &mut D) -> Result<u16, B::Error> {
    bus.write(sensor::ADDRESS, &[sensor::reg::CONTROL, sensor::START])
        .await?;
    delay.delay_ms(sensor::MEASURE_MS).await; // other tasks run meanwhile
    let mut data = [0u8; 2];
    bus.write_read(sensor::ADDRESS, &[sensor::reg::DATA], &mut data)
        .await?;
    Ok(u16::from_be_bytes(data))
}

// ---- Two tasks ---------------------------------------------------------------------------------
async fn blink_task(clock: &Clock) {
    let mut on = false;
    loop {
        on = !on;
        println!(
            "[{:>5} ms] led {}",
            clock.now(),
            if on { "on" } else { "off" }
        );
        clock.after(500).await;
    }
}

async fn sensor_task(clock: &Clock, world: &World) {
    let mut bus = AsyncBus {
        chip: SimSoilChip::new(world),
        clock,
    };
    let mut delay = AsyncDelay { clock };
    loop {
        let started = clock.now();
        match read_moisture(&mut bus, &mut delay).await {
            Ok(m) => println!(
                "[{:>5} ms] moisture {}.{}% (the read took {} ms of waiting)",
                clock.now(),
                m / 10,
                m % 10,
                clock.now() - started
            ),
            Err(err) => println!("[{:>5} ms] sensor error {err:?}", clock.now()),
        }
        world.advance_hour(); // speed up nature, so the numbers change
        clock.after(1200).await;
    }
}

// ---- The executor ---------------------------------------------------------------------------------
/// Polls every task, then jumps the clock to the earliest wake-up (the CPU
/// "sleeps" in between). Returns how many times it polled and slept.
///
/// A real executor only polls the tasks whose Waker was called; with two
/// tasks, polling both is simpler and does the same job.
fn run(
    tasks: &mut [Pin<&mut dyn Future<Output = ()>>],
    clock: &Clock,
    until_ms: u64,
) -> (u32, u32) {
    let mut cx = Context::from_waker(Waker::noop());
    let (mut polls, mut sleeps) = (0, 0);
    loop {
        clock.next_wake.set(None);
        for task in tasks.iter_mut() {
            polls += 1;
            // These tasks loop forever, so they never return Ready.
            let _ = task.as_mut().poll(&mut cx);
        }
        match clock.next_wake.get() {
            Some(at) if at <= until_ms => {
                sleeps += 1;
                clock.now_ms.set(at); // WFI ... the timer interrupt fires
            }
            _ => return (polls, sleeps),
        }
    }
}

fn main() {
    let clock = Clock {
        now_ms: Cell::new(0),
        next_wake: Cell::new(None),
    };
    let world = World::new(431, 2150);

    // `pin!` fixes each future in place on the stack: a future may hold
    // references into itself, so it must not move once it has started.
    let blink = pin!(blink_task(&clock));
    let sensor = pin!(sensor_task(&clock, &world));
    let mut tasks: [Pin<&mut dyn Future<Output = ()>>; 2] = [blink, sensor];

    let (polls, sleeps) = run(&mut tasks, &clock, 3000);
    println!(
        "\n3 simulated seconds: {polls} polls, {sleeps} sleeps. The CPU was busy only while polling."
    );
}
