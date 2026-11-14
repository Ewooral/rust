//! Exercise 6: write an I2C driver from a datasheet (pass 4 tests).
//!
//! The LUX-7 light sensor's datasheet, in short:
//!
//!     address  0x23
//!     ID       write [0x10], then read 1 byte: always 0x7A
//!     power on write [0x20, 0x01]
//!     data     write [0x14], then read 2 bytes: big-endian raw value
//!              (0xFFFF means "asleep: power me on first")
//!     lux      raw * 12 / 10   (do the maths in u32: 0xFFFE * 12 overflows u16)
//!
//! Implement `LightSensor` against embedded-hal's `I2c` trait:
//!   - `new`: read the ID with `write_read`; wrong ID -> `Error::WrongChip(id)`.
//!   - `power_on`: one `write`.
//!   - `read_lux`: one `write_read`; 0xFFFF -> `Error::Asleep`; else the lux.
//!
//! Every bus error becomes `Error::Bus(e)` (`.map_err(Error::Bus)?`).
//!
//! `MockBus` plays the chip and remembers the last bytes written, so the
//! tests can check you sent exactly what the datasheet says.
//!
//! Test: cargo test -p no_std_and_embedded --features exercises --bin ex06_light_sensor_driver

#![no_std]
#![allow(unused_variables, dead_code)]

use embedded_hal::i2c::{self, ErrorKind, ErrorType, I2c, Operation};

const ADDRESS: u8 = 0x23;
const ID_REGISTER: u8 = 0x10;
const EXPECTED_ID: u8 = 0x7A;
const POWER_ON: [u8; 2] = [0x20, 0x01];
const DATA_REGISTER: u8 = 0x14;
const ASLEEP: u16 = 0xFFFF;

#[derive(Debug, PartialEq, Eq)]
enum Error<E> {
    Bus(E),
    WrongChip(u8),
    Asleep,
}

struct LightSensor<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> LightSensor<I2C> {
    fn new(i2c: I2C) -> Result<LightSensor<I2C>, Error<I2C::Error>> {
        todo!()
    }

    fn power_on(&mut self) -> Result<(), Error<I2C::Error>> {
        todo!()
    }

    fn read_lux(&mut self) -> Result<u32, Error<I2C::Error>> {
        todo!()
    }
}

// ---- The mock chip -------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Nack;

impl i2c::Error for Nack {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

struct MockBus {
    id: u8,
    raw: u16,
    powered: bool,
    connected: bool,
    pointer: u8,
    last_write: [u8; 2],
    last_write_len: usize,
}

impl MockBus {
    fn new(raw: u16) -> MockBus {
        MockBus {
            id: EXPECTED_ID,
            raw,
            powered: false,
            connected: true,
            pointer: 0,
            last_write: [0; 2],
            last_write_len: 0,
        }
    }

    fn last_write(&self) -> &[u8] {
        &self.last_write[..self.last_write_len]
    }
}

impl ErrorType for MockBus {
    type Error = Nack;
}

impl I2c for MockBus {
    fn transaction(&mut self, address: u8, ops: &mut [Operation<'_>]) -> Result<(), Nack> {
        if address != ADDRESS || !self.connected {
            return Err(Nack);
        }
        for op in ops {
            match op {
                Operation::Write(bytes) => {
                    let n = bytes.len().min(2);
                    self.last_write[..n].copy_from_slice(&bytes[..n]);
                    self.last_write_len = n;
                    self.pointer = bytes.first().copied().unwrap_or(0);
                    if *bytes == POWER_ON {
                        self.powered = true;
                    }
                }
                Operation::Read(buffer) => {
                    let value = if self.pointer == ID_REGISTER {
                        u16::from(self.id) << 8
                    } else if self.powered {
                        self.raw
                    } else {
                        ASLEEP
                    };
                    let bytes = value.to_be_bytes();
                    for (slot, byte) in buffer.iter_mut().zip(bytes) {
                        *slot = byte;
                    }
                }
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
    let mut sensor = LightSensor::new(MockBus::new(500)).unwrap();
    host::say(format_args!("before power on: {:?}", sensor.read_lux()));
    sensor.power_on().unwrap();
    host::say(format_args!(
        "power on wrote:  {:02X?}",
        sensor.i2c.last_write()
    ));
    host::say(format_args!("after power on:  {:?} lux", sensor.read_lux()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_checks_the_id() {
        let sensor = LightSensor::new(MockBus::new(0)).unwrap();
        assert_eq!(sensor.i2c.last_write(), [ID_REGISTER]);
        let mut wrong = MockBus::new(0);
        wrong.id = 0x11;
        assert_eq!(LightSensor::new(wrong).err(), Some(Error::WrongChip(0x11)));
    }

    #[test]
    fn powers_on_then_reads_lux() {
        let mut sensor = LightSensor::new(MockBus::new(500)).unwrap();
        sensor.power_on().unwrap();
        assert_eq!(sensor.i2c.last_write(), POWER_ON);
        assert_eq!(sensor.read_lux(), Ok(600));
        assert_eq!(sensor.i2c.last_write(), [DATA_REGISTER]);
    }

    #[test]
    fn big_values_do_not_overflow_and_asleep_is_an_error() {
        let mut sensor = LightSensor::new(MockBus::new(0xFFFE)).unwrap();
        assert_eq!(sensor.read_lux(), Err(Error::Asleep));
        sensor.power_on().unwrap();
        assert_eq!(sensor.read_lux(), Ok(78_640));
    }

    #[test]
    fn bus_errors_are_wrapped() {
        let mut bus = MockBus::new(0);
        bus.connected = false;
        assert_eq!(LightSensor::new(bus).err(), Some(Error::Bus(Nack)));
        let mut sensor = LightSensor::new(MockBus::new(0)).unwrap();
        sensor.i2c.connected = false;
        assert_eq!(sensor.power_on(), Err(Error::Bus(Nack)));
        assert_eq!(sensor.read_lux(), Err(Error::Bus(Nack)));
    }
}
