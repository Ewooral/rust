//! The watering logic: a small state machine that drives the pump pin.
//!
//! Firmware is usually a loop: read the sensors, decide, set the outputs,
//! sleep, repeat. Keeping the "decide" part free of hardware (it only sees
//! an `OutputPin` and numbers) is what makes it testable on a PC.
//!
//! The rules:
//! - Smooth the readings: start on the average of the last 4, so one odd
//!   reading doesn't start the pump.
//! - **Hysteresis**: start below `start_below`, stop only above
//!   `stop_above`. One threshold would switch the pump on and off every
//!   hour around the line.
//! - **Safety cut-off**: never run for more than `max_run_hours`, even if
//!   the sensor says "still dry" (it might be broken, and the kitchen
//!   floor would flood).
//! - **Soak time**: after watering, wait `soak_hours` before deciding
//!   again, so the water can reach the sensor.
//! - **Fail safe**: when the sensor fails, switch the pump off.

use embedded_hal::digital::OutputPin;

use crate::ring::Ring;

/// The thresholds, all in per mille or whole hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Start watering when the smoothed moisture drops below this.
    pub start_below: u16,
    /// Stop once it rises above this.
    pub stop_above: u16,
    /// Never run the pump longer than this.
    pub max_run_hours: u16,
    /// After watering, wait this long before watering again.
    pub soak_hours: u16,
}

impl Config {
    /// Sensible values for a house plant.
    pub const HOUSE_PLANT: Config = Config {
        start_below: 300,
        stop_above: 450,
        max_run_hours: 4,
        soak_hours: 3,
    };
}

/// What the controller is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Waiting for the soil to dry out.
    Idle,
    /// The pump is on; `hours` so far.
    Watering {
        /// How long it has run.
        hours: u16,
    },
    /// Letting the water soak in.
    Soaking {
        /// Hours until it decides again.
        hours_left: u16,
    },
}

/// What happened in one step, for logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Nothing changed.
    Nothing,
    /// The soil was dry: pump on.
    PumpOn,
    /// The soil is wet enough: pump off.
    PumpOffWet,
    /// The pump ran too long: off, whatever the sensor says.
    PumpOffTimeout,
    /// The sensor failed while the pump was on: off, to be safe.
    PumpOffSensorFault,
    /// Soak time is over.
    Ready,
}

/// The watering controller. Owns the pump pin.
#[derive(Debug)]
pub struct Controller<P> {
    pump: P,
    config: Config,
    state: State,
    recent: Ring<u16, 4>,
    waterings: u16,
}

impl<P: OutputPin> Controller<P> {
    /// Takes the pump pin and makes sure the pump starts off.
    ///
    /// # Errors
    ///
    /// The pin's error, if it can't be set.
    pub fn new(mut pump: P, config: Config) -> Result<Controller<P>, P::Error> {
        pump.set_low()?;
        Ok(Controller {
            pump,
            config,
            state: State::Idle,
            recent: Ring::new(),
            waterings: 0,
        })
    }

    /// One step (an hour) with a fresh moisture reading.
    ///
    /// # Errors
    ///
    /// The pin's error, if it can't be set.
    pub fn step(&mut self, moisture: u16) -> Result<Event, P::Error> {
        self.recent.push(moisture);
        let smoothed = self.recent.average().unwrap_or(moisture);
        let (state, event) = match self.state {
            State::Idle if smoothed < self.config.start_below => {
                self.pump.set_high()?;
                self.waterings += 1;
                (State::Watering { hours: 1 }, Event::PumpOn)
            }
            State::Idle => (State::Idle, Event::Nothing),
            // Stop on the LATEST reading: the average would lag behind.
            State::Watering { .. } if moisture > self.config.stop_above => {
                self.stop()?;
                (self.soaking(), Event::PumpOffWet)
            }
            State::Watering { hours } if hours >= self.config.max_run_hours => {
                self.stop()?;
                (self.soaking(), Event::PumpOffTimeout)
            }
            State::Watering { hours } => (State::Watering { hours: hours + 1 }, Event::Nothing),
            State::Soaking { hours_left } if hours_left <= 1 => (State::Idle, Event::Ready),
            State::Soaking { hours_left } => (
                State::Soaking {
                    hours_left: hours_left - 1,
                },
                Event::Nothing,
            ),
        };
        self.state = state;
        Ok(event)
    }

    /// The sensor failed this step: never leave the pump running blind.
    ///
    /// # Errors
    ///
    /// The pin's error, if it can't be set.
    pub fn sensor_failed(&mut self) -> Result<Event, P::Error> {
        if let State::Watering { .. } = self.state {
            self.stop()?;
            self.state = self.soaking();
            return Ok(Event::PumpOffSensorFault);
        }
        Ok(Event::Nothing)
    }

    fn stop(&mut self) -> Result<(), P::Error> {
        self.pump.set_low()?;
        // Old readings are from before the water: start smoothing afresh.
        self.recent.clear();
        Ok(())
    }

    fn soaking(&self) -> State {
        State::Soaking {
            hours_left: self.config.soak_hours,
        }
    }

    /// The current state.
    pub fn state(&self) -> State {
        self.state
    }

    /// How many times the pump has been started.
    pub fn waterings(&self) -> u16 {
        self.waterings
    }

    /// Gives the pump pin back.
    pub fn release(self) -> P {
        self.pump
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimPump, World};

    fn controller(world: &World) -> Controller<SimPump<'_>> {
        Controller::new(SimPump::new(world), Config::HOUSE_PLANT).unwrap()
    }

    #[test]
    fn waters_when_dry_and_stops_when_wet() {
        let world = World::new(0, 2000);
        let mut c = controller(&world);
        assert_eq!(c.step(250).unwrap(), Event::PumpOn);
        assert!(world.pump_on.get());
        assert_eq!(c.step(380).unwrap(), Event::Nothing, "between the lines");
        assert_eq!(c.step(460).unwrap(), Event::PumpOffWet);
        assert!(!world.pump_on.get());
        assert_eq!(c.waterings(), 1);
    }

    #[test]
    fn one_odd_reading_does_not_start_the_pump() {
        let world = World::new(0, 2000);
        let mut c = controller(&world);
        for _ in 0..3 {
            c.step(500).unwrap();
        }
        assert_eq!(c.step(100).unwrap(), Event::Nothing, "average is 400");
        assert_eq!(c.state(), State::Idle);
    }

    #[test]
    fn hysteresis_keeps_the_pump_on_between_the_thresholds() {
        let world = World::new(0, 2000);
        let mut c = controller(&world);
        c.step(290).unwrap();
        for reading in [310, 330, 350] {
            assert_eq!(c.step(reading).unwrap(), Event::Nothing);
            assert!(world.pump_on.get(), "still on at {reading}");
        }
    }

    #[test]
    fn a_stuck_sensor_hits_the_safety_cutoff() {
        let world = World::new(0, 2000);
        let mut c = controller(&world);
        let events: Vec<Event> = (0..5).map(|_| c.step(100).unwrap()).collect();
        assert_eq!(events[0], Event::PumpOn);
        assert_eq!(events[4], Event::PumpOffTimeout);
        assert!(!world.pump_on.get());
    }

    #[test]
    fn soaks_before_deciding_again() {
        let world = World::new(0, 2000);
        let mut c = controller(&world);
        c.step(250).unwrap();
        c.step(500).unwrap(); // wet: off, soak 3 hours
        assert_eq!(c.step(100).unwrap(), Event::Nothing);
        assert_eq!(c.step(100).unwrap(), Event::Nothing);
        assert_eq!(c.step(100).unwrap(), Event::Ready);
        assert_eq!(c.step(100).unwrap(), Event::PumpOn);
        assert_eq!(c.waterings(), 2);
    }

    #[test]
    fn a_sensor_fault_turns_the_pump_off() {
        let world = World::new(0, 2000);
        let mut c = controller(&world);
        assert_eq!(c.sensor_failed().unwrap(), Event::Nothing);
        c.step(200).unwrap();
        assert_eq!(c.sensor_failed().unwrap(), Event::PumpOffSensorFault);
        assert!(!world.pump_on.get());
        assert_eq!(c.release().switches, 2);
    }
}
