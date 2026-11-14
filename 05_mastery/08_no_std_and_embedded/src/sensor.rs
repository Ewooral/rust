//! A driver for the (pretend) **SM-01 soil moisture sensor**, an I2C chip.
//!
//! The driver is generic over `embedded_hal::i2c::I2c` and
//! `embedded_hal::delay::DelayNs`, so the *same code* runs on an STM32, an
//! RP2040, an ESP32, or on your PC against [`crate::sim::SimSoilChip`].
//! That is the whole point of embedded-hal: drivers are written once,
//! against traits, and every board's HAL implements the traits.
//!
//! The chip's "datasheet":
//!
//! | Register | Name | Meaning |
//! |---|---|---|
//! | `0x00` | `CHIP_ID` | always reads `0x5A` |
//! | `0x01` | `CONTROL` | write `0x01` to start a measurement (takes 5 ms) |
//! | `0x02..=0x03` | `MOISTURE` | big-endian `u16`, 0 (dry) to 1000 (soaked) per mille |
//! | `0x04..=0x05` | `TEMP` | big-endian `i16`, hundredths of a degree Celsius |
//!
//! Reading continues into the next register automatically, so one 4-byte
//! read from `0x02` returns moisture and temperature together.

use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

/// The chip's 7-bit I2C address.
pub const ADDRESS: u8 = 0x36;
/// What `CHIP_ID` reads on a real SM-01.
pub const CHIP_ID: u8 = 0x5A;

/// Register addresses.
pub mod reg {
    /// Identifies the chip.
    pub const CHIP_ID: u8 = 0x00;
    /// Write [`START`](super::START) here to measure.
    pub const CONTROL: u8 = 0x01;
    /// First data register: moisture, then temperature.
    pub const DATA: u8 = 0x02;
}

/// The command that starts a measurement.
pub const START: u8 = 0x01;
/// How long a measurement takes, from the datasheet.
pub const MEASURE_MS: u32 = 5;

/// One measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    /// 0 (bone dry) to 1000 (soaked).
    pub moisture_permille: u16,
    /// Hundredths of a degree: 2150 is 21.50 °C.
    pub temp_centi: i16,
}

/// What can go wrong. Generic over the bus's own error type `E`, so the
/// caller still sees exactly what the I2C peripheral reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// The I2C bus failed (no answer, lost arbitration, ...).
    Bus(E),
    /// Something answered, but it isn't an SM-01.
    WrongChip(u8),
    /// The chip sent a moisture value over 1000: a faulty sensor.
    OutOfRange(u16),
}

/// The driver. It **owns** its bus and delay: nothing else can talk to the
/// chip behind its back. [`SoilSensor::release`] gives them back.
#[derive(Debug)]
pub struct SoilSensor<I2C, D> {
    i2c: I2C,
    delay: D,
}

impl<I2C: I2c, D: DelayNs> SoilSensor<I2C, D> {
    /// Checks the chip is there and is really an SM-01.
    ///
    /// # Errors
    ///
    /// `Bus` if nothing answers, `WrongChip` if the ID is wrong.
    pub fn new(mut i2c: I2C, delay: D) -> Result<Self, Error<I2C::Error>> {
        let mut id = [0u8; 1];
        i2c.write_read(ADDRESS, &[reg::CHIP_ID], &mut id)
            .map_err(Error::Bus)?;
        if id[0] != CHIP_ID {
            return Err(Error::WrongChip(id[0]));
        }
        Ok(SoilSensor { i2c, delay })
    }

    /// Starts a measurement, waits for it, and reads the result.
    ///
    /// # Errors
    ///
    /// `Bus` for bus failures, `OutOfRange` for impossible values.
    pub fn read(&mut self) -> Result<Reading, Error<I2C::Error>> {
        self.i2c
            .write(ADDRESS, &[reg::CONTROL, START])
            .map_err(Error::Bus)?;
        self.delay.delay_ms(MEASURE_MS);
        let mut data = [0u8; 4];
        self.i2c
            .write_read(ADDRESS, &[reg::DATA], &mut data)
            .map_err(Error::Bus)?;
        let moisture_permille = u16::from_be_bytes([data[0], data[1]]);
        if moisture_permille > 1000 {
            return Err(Error::OutOfRange(moisture_permille));
        }
        Ok(Reading {
            moisture_permille,
            temp_centi: i16::from_be_bytes([data[2], data[3]]),
        })
    }

    /// Gives the bus and the delay back, for another driver to use.
    pub fn release(self) -> (I2C, D) {
        (self.i2c, self.delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimDelay, SimI2cError, SimSoilChip, World};

    #[test]
    fn checks_the_chip_id_then_reads() {
        let world = World::new(420, 2150);
        let mut sensor = SoilSensor::new(SimSoilChip::new(&world), SimDelay::default()).unwrap();
        assert_eq!(
            sensor.read(),
            Ok(Reading {
                moisture_permille: 420,
                temp_centi: 2150
            })
        );
        let (chip, delay) = sensor.release();
        assert_eq!(chip.transactions, 3, "id, start, data");
        assert_eq!(delay.total_ns, 5_000_000, "waited 5 ms for the measurement");
    }

    #[test]
    fn negative_temperatures_survive_the_bytes() {
        let world = World::new(10, -1250);
        let mut sensor = SoilSensor::new(SimSoilChip::new(&world), SimDelay::default()).unwrap();
        assert_eq!(sensor.read().unwrap().temp_centi, -1250);
    }

    #[test]
    fn wrong_chip_and_missing_chip_are_errors() {
        let world = World::new(420, 2150);
        let mut chip = SimSoilChip::new(&world);
        chip.chip_id = 0x77;
        assert_eq!(
            SoilSensor::new(chip, SimDelay::default()).unwrap_err(),
            Error::WrongChip(0x77)
        );
        world.sensor_unplugged.set(true);
        assert_eq!(
            SoilSensor::new(SimSoilChip::new(&world), SimDelay::default()).unwrap_err(),
            Error::Bus(SimI2cError::NoAcknowledge)
        );
    }

    #[test]
    fn impossible_values_are_rejected() {
        let world = World::new(420, 2150);
        let mut sensor = SoilSensor::new(SimSoilChip::new(&world), SimDelay::default()).unwrap();
        world.moisture.set(4000);
        assert_eq!(sensor.read(), Err(Error::OutOfRange(4000)));
    }
}
