//! Simulated hardware, so the firmware runs and is tested on your PC.
//!
//! Each simulated part implements the same embedded-hal trait a real
//! board's HAL would: [`SimSoilChip`] is an `I2c` bus with an SM-01 on it,
//! [`SimPump`] is an `OutputPin`, [`SimDelay`] is a `DelayNs`. The drivers
//! can't tell the difference.
//!
//! The simulation itself is `no_std` too: the shared [`World`] uses `Cell`
//! (interior mutability without a heap or locks), so the chip, the pump and
//! the test can all look at the same pot of soil.

use core::cell::Cell;
use core::convert::Infallible;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{self, OutputPin};
use embedded_hal::i2c::{self, ErrorKind, I2c, NoAcknowledgeSource, Operation};

use crate::sensor;

/// The pretend physical world: a pot of soil, a thermometer and a pump.
#[derive(Debug)]
pub struct World {
    /// Soil moisture, 0 to 1000 per mille.
    pub moisture: Cell<u16>,
    /// Air temperature in hundredths of a degree.
    pub temp_centi: Cell<i16>,
    /// Is the pump running? Set by [`SimPump`].
    pub pump_on: Cell<bool>,
    /// Pull the sensor's cable out: every I2C transaction fails.
    pub sensor_unplugged: Cell<bool>,
}

impl World {
    /// A pot with this much moisture, at this temperature.
    pub const fn new(moisture: u16, temp_centi: i16) -> World {
        World {
            moisture: Cell::new(moisture),
            temp_centi: Cell::new(temp_centi),
            pump_on: Cell::new(false),
            sensor_unplugged: Cell::new(false),
        }
    }

    /// One hour passes: water evaporates (faster when it's warm), and a
    /// running pump adds water.
    pub fn advance_hour(&self) {
        let warmth = (self.temp_centi.get() / 100 - 15).max(0) as u16;
        let evaporation = 6 + warmth / 2;
        let pumped = if self.pump_on.get() { 70 } else { 0 };
        let moisture = self.moisture.get().saturating_sub(evaporation) + pumped;
        self.moisture.set(moisture.min(1000));
    }
}

/// The simulated bus's error: nobody answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimI2cError {
    /// No device acknowledged the address.
    NoAcknowledge,
}

// Every bus error must say what *kind* of error it is, so generic code can
// react without knowing the HAL.
impl i2c::Error for SimI2cError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address)
    }
}

/// An I2C bus with one SM-01 soil sensor on it, reading from a [`World`].
#[derive(Debug)]
pub struct SimSoilChip<'w> {
    world: &'w World,
    /// What the `CHIP_ID` register says. Change it to fake a different chip.
    pub chip_id: u8,
    /// How many transactions the driver has made.
    pub transactions: u32,
    pointer: u8,
    latched: [u8; 4],
}

impl<'w> SimSoilChip<'w> {
    /// A chip that senses `world`.
    pub fn new(world: &'w World) -> SimSoilChip<'w> {
        SimSoilChip {
            world,
            chip_id: sensor::CHIP_ID,
            transactions: 0,
            pointer: 0,
            latched: [0; 4],
        }
    }

    fn register(&self, address: u8) -> u8 {
        match address {
            sensor::reg::CHIP_ID => self.chip_id,
            sensor::reg::CONTROL => 0,
            a @ 0x02..=0x05 => self.latched[usize::from(a - 0x02)],
            _ => 0xFF,
        }
    }
}

impl i2c::ErrorType for SimSoilChip<'_> {
    type Error = SimI2cError;
}

impl I2c for SimSoilChip<'_> {
    // The only required method. `read`, `write` and `write_read` have
    // default bodies that build an `operations` list and call this.
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), SimI2cError> {
        self.transactions += 1;
        if address != sensor::ADDRESS || self.world.sensor_unplugged.get() {
            return Err(SimI2cError::NoAcknowledge);
        }
        for operation in operations {
            match operation {
                Operation::Write(bytes) => {
                    if let Some((&register, rest)) = bytes.split_first() {
                        self.pointer = register;
                        if register == sensor::reg::CONTROL && rest.first() == Some(&sensor::START)
                        {
                            let [m1, m2] = self.world.moisture.get().to_be_bytes();
                            let [t1, t2] = self.world.temp_centi.get().to_be_bytes();
                            self.latched = [m1, m2, t1, t2];
                        }
                    }
                }
                Operation::Read(buffer) => {
                    for byte in buffer.iter_mut() {
                        *byte = self.register(self.pointer);
                        self.pointer = self.pointer.wrapping_add(1);
                    }
                }
            }
        }
        Ok(())
    }
}

/// A water pump on a GPIO pin: high is on.
#[derive(Debug)]
pub struct SimPump<'w> {
    world: &'w World,
    /// How many times it has been switched on or off.
    pub switches: u32,
}

impl<'w> SimPump<'w> {
    /// A pump that waters `world`.
    pub fn new(world: &'w World) -> SimPump<'w> {
        SimPump { world, switches: 0 }
    }

    fn set(&mut self, on: bool) {
        if self.world.pump_on.get() != on {
            self.switches += 1;
            self.world.pump_on.set(on);
        }
    }
}

// `Infallible`: setting a GPIO pin can't fail, so the error type is one
// that has no values at all. embedded-hal implements its Error trait.
impl digital::ErrorType for SimPump<'_> {
    type Error = Infallible;
}

impl OutputPin for SimPump<'_> {
    fn set_low(&mut self) -> Result<(), Infallible> {
        self.set(false);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Infallible> {
        self.set(true);
        Ok(())
    }
}

/// A delay that doesn't wait: it adds up how long it *would* have waited.
#[derive(Debug, Default)]
pub struct SimDelay {
    /// Total time "waited", in nanoseconds.
    pub total_ns: u64,
}

impl DelayNs for SimDelay {
    // The only required method; `delay_us` and `delay_ms` call it.
    fn delay_ns(&mut self, ns: u32) {
        self.total_ns += u64::from(ns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soil_dries_faster_when_warm_and_the_pump_waters_it() {
        let cool = World::new(500, 1500);
        let warm = World::new(500, 2500);
        cool.advance_hour();
        warm.advance_hour();
        assert_eq!(cool.moisture.get(), 494);
        assert_eq!(warm.moisture.get(), 489);
        warm.pump_on.set(true);
        warm.advance_hour();
        assert_eq!(warm.moisture.get(), 548);
    }

    #[test]
    fn moisture_stays_in_range() {
        let world = World::new(3, 2000);
        world.advance_hour();
        assert_eq!(world.moisture.get(), 0);
        let world = World::new(990, 2000);
        world.pump_on.set(true);
        world.advance_hour();
        assert_eq!(world.moisture.get(), 1000);
    }

    #[test]
    fn the_chip_answers_only_its_own_address() {
        let world = World::new(500, 2000);
        let mut chip = SimSoilChip::new(&world);
        let mut id = [0u8];
        chip.write_read(0x36, &[0x00], &mut id).unwrap();
        assert_eq!(id, [0x5A]);
        assert_eq!(chip.read(0x40, &mut id), Err(SimI2cError::NoAcknowledge));
    }

    #[test]
    fn the_pump_counts_real_switches_only() {
        let world = World::new(500, 2000);
        let mut pump = SimPump::new(&world);
        pump.set_high().unwrap();
        pump.set_high().unwrap();
        pump.set_low().unwrap();
        assert_eq!(pump.switches, 2);
        assert!(!world.pump_on.get());
    }
}
