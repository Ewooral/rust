//! Solution to exercise 3: make it compile (E0277).
//!
//! A driver for a small I2C display controller. Its error type wraps the
//! bus's own error, `Error::Bus(E)`, so callers can still see what the HAL
//! reported. But `?` only converts errors through the `From` trait, and
//! nothing says how to turn an `I2C::Error` into an `Error<I2C::Error>`.
//!
//! Fix `status` so the bus error becomes `Error::Bus`. Two good ways:
//!   a) `.map_err(Error::Bus)?` on the bus call (a variant is a function!);
//!   b) `impl<E> From<E> for Error<E>`, so plain `?` works everywhere.
//! Most embedded drivers pick (a): it keeps the conversion visible.
//!
//! Run: cargo run -p no_std_and_embedded --features solutions --bin sol03_fix_bus_error_conversion

#![no_std]

use embedded_hal::i2c::{self, ErrorKind, ErrorType, I2c, Operation};

const ADDRESS: u8 = 0x3C;
const STATUS_REGISTER: u8 = 0x00;
const BUSY: u8 = 0x80;

#[derive(Debug, PartialEq)]
enum Error<E> {
    Bus(E),
    Busy,
}

// Fix (b) also compiles: plain `?` would then convert any E into Error::Bus.
//     impl<E> From<E> for Error<E> {
//         fn from(e: E) -> Self {
//             Error::Bus(e)
//         }
//     }

struct DisplayController<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> DisplayController<I2C> {
    fn status(&mut self) -> Result<u8, Error<I2C::Error>> {
        let mut status = [0u8];
        self.i2c
            .write_read(ADDRESS, &[STATUS_REGISTER], &mut status)
            .map_err(Error::Bus)?; // fix (a): a tuple variant is a function E -> Error<E>
        if status[0] & BUSY != 0 {
            return Err(Error::Busy);
        }
        Ok(status[0])
    }
}

// ---- A pretend bus, for the PC ------------------------------------------------------------
#[derive(Debug, PartialEq)]
struct NoAnswer;

impl i2c::Error for NoAnswer {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

struct FakeBus {
    status: u8,
    connected: bool,
}

impl ErrorType for FakeBus {
    type Error = NoAnswer;
}

impl I2c for FakeBus {
    fn transaction(&mut self, _address: u8, ops: &mut [Operation<'_>]) -> Result<(), NoAnswer> {
        if !self.connected {
            return Err(NoAnswer);
        }
        for op in ops {
            if let Operation::Read(buffer) = op {
                buffer.fill(self.status);
            }
        }
        Ok(())
    }
}

mod host {
    extern crate std;

    pub fn say(line: core::fmt::Arguments) {
        std::println!("{line}");
    }
}

fn main() {
    for (status, connected) in [(0x01, true), (0x81, true), (0x01, false)] {
        let mut display = DisplayController {
            i2c: FakeBus { status, connected },
        };
        host::say(format_args!(
            "status {status:#04x}, connected {connected}: {:?}",
            display.status()
        ));
    }
}
