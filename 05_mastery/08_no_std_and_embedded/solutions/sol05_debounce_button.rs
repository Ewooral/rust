//! Solution to exercise 5: debounce a button (pass 3 tests).
//!
//! A mechanical button doesn't switch cleanly: for a few milliseconds the
//! contacts bounce, and the pin reads low, high, low, high... A firmware
//! loop that samples it every millisecond would see five presses.
//!
//! `Debouncer` only believes a new level once it has read it `needed` times
//! IN A ROW. The button is wired active-low: LOW means pressed.
//!
//! Implement `update`, called once per sample:
//!   1. read the pin (`self.pin.is_low()?` is true when pressed);
//!   2. if that matches the state we already believe, reset `count` to 0
//!      and return `Ok(None)`;
//!   3. otherwise add 1 to `count`; once it reaches `needed`, flip
//!      `pressed`, reset `count`, and return the edge:
//!      `Some(Edge::Pressed)` or `Some(Edge::Released)`.
//!
//! It works with ANY `InputPin`: the tests use `ScriptedPin`, a board
//! would pass its HAL's pin type.
//!
//! Test: cargo test -p no_std_and_embedded --features solutions --bin sol05_debounce_button

#![no_std]

use core::convert::Infallible;

use embedded_hal::digital::{ErrorType, InputPin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Pressed,
    Released,
}

struct Debouncer<P> {
    pin: P,
    pressed: bool,
    count: u8,
    needed: u8,
}

impl<P: InputPin> Debouncer<P> {
    fn new(pin: P, needed: u8) -> Debouncer<P> {
        Debouncer {
            pin,
            pressed: false,
            count: 0,
            needed,
        }
    }

    fn update(&mut self) -> Result<Option<Edge>, P::Error> {
        let pressed_now = self.pin.is_low()?;
        if pressed_now == self.pressed {
            self.count = 0;
            return Ok(None);
        }
        self.count += 1;
        if self.count < self.needed {
            return Ok(None);
        }
        self.pressed = pressed_now;
        self.count = 0;
        Ok(Some(if pressed_now {
            Edge::Pressed
        } else {
            Edge::Released
        }))
    }
}

/// A pin that plays back a script of levels: `false` is low (pressed).
/// After the script it stays high (released).
struct ScriptedPin {
    levels: &'static [bool],
    next: usize,
}

impl ErrorType for ScriptedPin {
    type Error = Infallible;
}

impl InputPin for ScriptedPin {
    fn is_high(&mut self) -> Result<bool, Infallible> {
        let level = self.levels.get(self.next).copied().unwrap_or(true);
        self.next += 1;
        Ok(level)
    }

    fn is_low(&mut self) -> Result<bool, Infallible> {
        Ok(!self.is_high()?)
    }
}

mod host {
    extern crate std;

    pub fn say(line: core::fmt::Arguments) {
        std::println!("{line}");
    }
}

/// One bouncy press: bounce, hold, bounce, release.
const BOUNCY_PRESS: &[bool] = &[
    true, false, true, false, false, false, false, false, true, false, true, true, true, true,
];

fn main() {
    let mut button = Debouncer::new(
        ScriptedPin {
            levels: BOUNCY_PRESS,
            next: 0,
        },
        3,
    );
    for (sample, level) in BOUNCY_PRESS.iter().enumerate() {
        let edge = button.update().unwrap();
        host::say(format_args!(
            "sample {sample:>2}: pin {} -> {edge:?}",
            if *level { "high" } else { "LOW " }
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(levels: &'static [bool], needed: u8) -> [Option<Edge>; 16] {
        let mut button = Debouncer::new(ScriptedPin { levels, next: 0 }, needed);
        core::array::from_fn(|_| button.update().unwrap())
    }

    #[test]
    fn one_bouncy_press_is_one_press_and_one_release() {
        let seen = edges(BOUNCY_PRESS, 3);
        let presses = seen.iter().filter(|e| **e == Some(Edge::Pressed)).count();
        let releases = seen.iter().filter(|e| **e == Some(Edge::Released)).count();
        assert_eq!((presses, releases), (1, 1));
        assert_eq!(seen[5], Some(Edge::Pressed), "3rd low in a row");
        assert_eq!(seen[12], Some(Edge::Released), "3rd high in a row");
    }

    #[test]
    fn short_glitches_are_ignored() {
        let seen = edges(&[true, false, true, false, false, true, true], 3);
        assert!(seen.iter().all(Option::is_none), "{seen:?}");
    }

    #[test]
    fn needed_1_reacts_at_once() {
        let seen = edges(&[false, true], 1);
        assert_eq!(seen[0], Some(Edge::Pressed));
        assert_eq!(seen[1], Some(Edge::Released));
    }
}
