//! embedded-hal: write the code once, against traits, and run it on any board.
//! Run: cargo run -p no_std_and_embedded --example hal_traits
//!
//! The layers of a Rust embedded project, from the metal up:
//!
//!   PAC   (peripheral access crate)  registers as Rust types, generated from
//!          the chip maker's description file. One per chip: `stm32f4`, `rp2040-pac`.
//!   HAL   (hardware abstraction layer) safe drivers for that chip's pins,
//!          timers, I2C... that IMPLEMENT the embedded-hal traits: `stm32f4xx-hal`,
//!          `rp2040-hal`, `esp-hal`, `embassy-stm32`, `embassy-rp`.
//!   embedded-hal                       the traits themselves: OutputPin, InputPin,
//!          I2c, SpiDevice, DelayNs. No code for any chip.
//!   drivers + your app                 generic over the traits, like `blink` below.
//!
//! Here the "boards" are two structs on your PC. `blink` can't tell.

use std::convert::Infallible;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, PinState, StatefulOutputPin};

/// Generic over ANY output pin and ANY delay: this exact function would
/// compile for an STM32 or an RP2040 with that HAL's pin and delay types.
fn blink<P: OutputPin, D: DelayNs>(led: &mut P, delay: &mut D, times: u32) -> Result<(), P::Error> {
    for _ in 0..times {
        led.set_high()?;
        delay.delay_ms(250);
        led.set_low()?;
        delay.delay_ms(250);
    }
    Ok(())
}

/// Wait for a button, then flip the LED: `toggle` comes from StatefulOutputPin,
/// which also lets you ask a pin what it was last set to.
fn toggle_on_press<B: InputPin, L: StatefulOutputPin>(
    button: &mut B,
    led: &mut L,
) -> Result<bool, &'static str> {
    // Buttons are usually wired "active low": pressed connects the pin to ground.
    if button.is_low().map_err(|_| "button")? {
        led.toggle().map_err(|_| "led")?;
        return Ok(true);
    }
    Ok(false)
}

// ---- Board 1: an LED that prints, and a delay that really sleeps (briefly) -----------------
struct ConsoleLed {
    name: &'static str,
    on: bool,
}

impl ErrorType for ConsoleLed {
    type Error = Infallible; // a GPIO write can't fail
}

impl OutputPin for ConsoleLed {
    fn set_low(&mut self) -> Result<(), Infallible> {
        self.on = false;
        println!("  {} off", self.name);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Infallible> {
        self.on = true;
        println!("  {} ON", self.name);
        Ok(())
    }
}

impl StatefulOutputPin for ConsoleLed {
    fn is_set_high(&mut self) -> Result<bool, Infallible> {
        Ok(self.on)
    }
    fn is_set_low(&mut self) -> Result<bool, Infallible> {
        Ok(!self.on)
    }
}

/// Sleeps for 1/50th of the asked time, so the demo is quick.
struct FastClock;

impl DelayNs for FastClock {
    fn delay_ns(&mut self, ns: u32) {
        std::thread::sleep(std::time::Duration::from_nanos(u64::from(ns / 50)));
    }
}

// ---- Board 2: a pin that records, and a delay that only counts ---------------------------------
#[derive(Default)]
struct RecordingPin {
    history: Vec<PinState>,
}

impl ErrorType for RecordingPin {
    type Error = Infallible;
}

impl OutputPin for RecordingPin {
    fn set_low(&mut self) -> Result<(), Infallible> {
        self.history.push(PinState::Low);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Infallible> {
        self.history.push(PinState::High);
        Ok(())
    }
}

#[derive(Default)]
struct CountingDelay {
    total_ms: u32,
}

impl DelayNs for CountingDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.total_ms += ns / 1_000_000;
    }
    // Override the default (which would call delay_ns 1000 times per ms in
    // chunks) to count whole milliseconds exactly.
    fn delay_ms(&mut self, ms: u32) {
        self.total_ms += ms;
    }
}

/// A button that reads from a script: false = low = pressed.
struct ScriptedButton {
    levels: &'static [bool],
    next: usize,
}

impl ErrorType for ScriptedButton {
    type Error = Infallible;
}

impl InputPin for ScriptedButton {
    fn is_high(&mut self) -> Result<bool, Infallible> {
        let level = self.levels.get(self.next).copied().unwrap_or(true);
        self.next += 1;
        Ok(level)
    }
    fn is_low(&mut self) -> Result<bool, Infallible> {
        self.is_high().map(|high| !high)
    }
}

fn main() {
    println!("Board 1: console LED, real (but fast) sleeps");
    let mut led = ConsoleLed {
        name: "green LED",
        on: false,
    };
    blink(&mut led, &mut FastClock, 2).unwrap();

    println!("\nBoard 2: recording pin, counting delay");
    let mut pin = RecordingPin::default();
    let mut delay = CountingDelay::default();
    blink(&mut pin, &mut delay, 3).unwrap();
    println!("  pin history: {:?}", pin.history);
    println!("  waited:      {} ms", delay.total_ms);

    println!("\nA button wired active-low, polled 5 times:");
    let mut button = ScriptedButton {
        levels: &[true, false, true, true, false],
        next: 0,
    };
    for poll in 1..=5 {
        let pressed = toggle_on_press(&mut button, &mut led).unwrap();
        println!("  poll {poll}: pressed = {pressed}");
    }
}
