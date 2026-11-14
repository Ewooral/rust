//! The mini project: firmware for a **self-watering plant pot**, written as
//! a `no_std` library and tested on your PC.
//!
//! | Module | Lesson topic |
//! |---|---|
//! | [`ring`] | fixed-capacity collections with const generics |
//! | [`sensor`] | an I2C driver written against embedded-hal traits |
//! | [`controller`] | firmware logic over an `OutputPin`, testable anywhere |
//! | [`report`] | Serde and postcard in `no_std`, formatting without `String` |
//! | [`sim`] | simulated hardware (also `no_std`) for tests and the PC demo |
//!
//! The line below is the usual trick for `no_std` libraries: the crate is
//! `no_std` for real builds, but `cargo test` builds it WITH std, because
//! the test harness itself needs threads, printing and panics that unwind.
#![cfg_attr(not(test), no_std)]

// `alloc` is the part of std that only needs a heap: Vec, String, Box, Rc,
// BTreeMap, format!. A no_std crate must ask for it by name.
#[cfg(feature = "alloc")]
extern crate alloc;

pub mod controller;
pub mod report;
pub mod ring;
pub mod sensor;
pub mod sim;

/// Ties the parts together: one hour of firmware.
///
/// Reads the sensor, lets the controller decide, and returns the status to
/// report. A failed read switches the pump off and is counted, but never
/// stops the loop: firmware has nobody to show a panic message to.
///
/// # Errors
///
/// Only the pump pin's error: a sensor failure is handled, not returned.
pub fn run_hour<I2C, D, P>(
    sensor: &mut sensor::SoilSensor<I2C, D>,
    controller: &mut controller::Controller<P>,
    status: &mut report::Status,
) -> Result<controller::Event, P::Error>
where
    I2C: embedded_hal::i2c::I2c,
    D: embedded_hal::delay::DelayNs,
    P: embedded_hal::digital::OutputPin,
{
    status.uptime_hours += 1;
    let event = match sensor.read() {
        Ok(reading) => {
            status.moisture_permille = reading.moisture_permille;
            status.temp_centi = reading.temp_centi;
            controller.step(reading.moisture_permille)?
        }
        Err(_) => {
            status.sensor_faults += 1;
            controller.sensor_failed()?
        }
    };
    status.pump_on = matches!(controller.state(), controller::State::Watering { .. });
    status.waterings = controller.waterings();
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::{Config, Controller, Event};
    use crate::report::Status;
    use crate::sensor::SoilSensor;
    use crate::sim::{SimDelay, SimPump, SimSoilChip, World};

    #[test]
    fn a_dry_pot_gets_watered_back_into_range() {
        let world = World::new(320, 2200);
        let mut sensor = SoilSensor::new(SimSoilChip::new(&world), SimDelay::default()).unwrap();
        let mut controller = Controller::new(SimPump::new(&world), Config::HOUSE_PLANT).unwrap();
        let mut status = Status {
            uptime_hours: 0,
            moisture_permille: 0,
            temp_centi: 0,
            pump_on: false,
            waterings: 0,
            sensor_faults: 0,
        };
        let mut events = Vec::new();
        for _ in 0..48 {
            events.push(run_hour(&mut sensor, &mut controller, &mut status).unwrap());
            world.advance_hour();
        }
        assert!(events.contains(&Event::PumpOn));
        assert!(events.contains(&Event::PumpOffWet));
        assert!(!events.contains(&Event::PumpOffTimeout));
        assert!(status.waterings >= 2, "{status:?}");
        assert!((200..=600).contains(&world.moisture.get()));
        assert_eq!(status.uptime_hours, 48);
    }

    #[test]
    fn an_unplugged_sensor_stops_the_pump_and_is_counted() {
        let world = World::new(100, 2200);
        let mut sensor = SoilSensor::new(SimSoilChip::new(&world), SimDelay::default()).unwrap();
        let mut controller = Controller::new(SimPump::new(&world), Config::HOUSE_PLANT).unwrap();
        let mut status = Status {
            uptime_hours: 0,
            moisture_permille: 0,
            temp_centi: 0,
            pump_on: false,
            waterings: 0,
            sensor_faults: 0,
        };
        assert_eq!(
            run_hour(&mut sensor, &mut controller, &mut status),
            Ok(Event::PumpOn)
        );
        world.sensor_unplugged.set(true);
        assert_eq!(
            run_hour(&mut sensor, &mut controller, &mut status),
            Ok(Event::PumpOffSensorFault)
        );
        assert!(!world.pump_on.get());
        assert_eq!(status.sensor_faults, 1);
        assert!(!status.pump_on);
    }
}
