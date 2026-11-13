//! Solution to exercise 4: methods on an enum.
//!
//! Red -> Green -> Yellow -> Red. Red lasts 60 s, Green 45 s, Yellow 5 s.
//!
//! Test: cargo test -p enums_and_pattern_matching --features solutions --bin sol04_traffic_light

#[derive(Debug, Clone, Copy, PartialEq)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    /// The light that comes after this one.
    fn next(self) -> Self {
        match self {
            TrafficLight::Red => TrafficLight::Green,
            TrafficLight::Green => TrafficLight::Yellow,
            TrafficLight::Yellow => TrafficLight::Red,
        }
    }

    /// How long this light stays on, in seconds.
    fn duration_secs(&self) -> u32 {
        match self {
            TrafficLight::Red => 60,
            TrafficLight::Green => 45,
            TrafficLight::Yellow => 5,
        }
    }

    /// May a car drive through?
    fn can_go(&self) -> bool {
        matches!(self, TrafficLight::Green)
    }
}

fn main() {
    let mut light = TrafficLight::Red;
    for _ in 0..4 {
        println!(
            "{light:?} for {}s, go: {}",
            light.duration_secs(),
            light.can_go()
        );
        light = light.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TrafficLight::*;

    #[test]
    fn cycles_in_order() {
        assert_eq!(Red.next(), Green);
        assert_eq!(Green.next(), Yellow);
        assert_eq!(Yellow.next(), Red);
    }

    #[test]
    fn a_full_cycle_takes_110_seconds() {
        let mut light = Red;
        let mut total = 0;
        for _ in 0..3 {
            total += light.duration_secs();
            light = light.next();
        }
        assert_eq!(total, 110);
        assert_eq!(light, Red);
    }

    #[test]
    fn only_green_means_go() {
        assert!(Green.can_go());
        assert!(!Red.can_go());
        assert!(!Yellow.can_go());
    }
}
