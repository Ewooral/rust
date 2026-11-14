//! Exercise 5: choose what happens on overflow (Beyond the Book).
//!
//! Plain `+`, `-` and `*` panic on overflow in a debug build and wrap in a
//! release build. Implement each function with the integer method that says
//! what should happen instead:
//!
//! - `add_health`: health is a u8 that stops at 255 (`saturating_*`).
//! - `seats_left`: never goes below 0 (`saturating_*`).
//! - `total_price`: `None` if the result doesn't fit in a u32 (`checked_*`).
//! - `next_frame`: an animation counter that goes 254, 255, 0, 1, ... (`wrapping_*`).
//!
//! Test: cargo test -p variables_and_data_types --features exercises --bin ex05_safe_arithmetic

#![allow(unused_variables)]

fn add_health(health: u8, potion: u8) -> u8 {
    todo!("{health} + {potion}, stopping at 255")
}

fn seats_left(seats: u32, booked: u32) -> u32 {
    todo!("{seats} - {booked}, stopping at 0")
}

fn total_price(unit_price: u32, quantity: u32) -> Option<u32> {
    todo!("{unit_price} * {quantity}, or None on overflow")
}

fn next_frame(frame: u8) -> u8 {
    todo!("{frame} + 1, wrapping from 255 to 0")
}

fn main() {
    println!("health: {}", add_health(250, 10));
    println!("seats left: {}", seats_left(5, 8));
    println!("price: {:?}", total_price(u32::MAX, 2));
    println!("frame after 255: {}", next_frame(255));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_stops_at_255() {
        assert_eq!(add_health(100, 10), 110);
        assert_eq!(add_health(250, 10), 255);
        assert_eq!(add_health(255, 255), 255);
    }

    #[test]
    fn seats_never_go_negative() {
        assert_eq!(seats_left(10, 3), 7);
        assert_eq!(seats_left(5, 8), 0);
    }

    #[test]
    fn price_is_none_when_it_overflows() {
        assert_eq!(total_price(1_000, 3), Some(3_000));
        assert_eq!(total_price(u32::MAX, 1), Some(u32::MAX));
        assert_eq!(total_price(u32::MAX, 2), None);
    }

    #[test]
    fn frames_wrap_around() {
        assert_eq!(next_frame(0), 1);
        assert_eq!(next_frame(254), 255);
        assert_eq!(next_frame(255), 0);
    }
}
