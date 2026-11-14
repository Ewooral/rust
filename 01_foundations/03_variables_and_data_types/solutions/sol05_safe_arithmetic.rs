//! Solution to exercise 5: choose what happens on overflow (Beyond the Book).
//!
//! Test: cargo test -p variables_and_data_types --features solutions --bin sol05_safe_arithmetic

fn add_health(health: u8, potion: u8) -> u8 {
    health.saturating_add(potion)
}

fn seats_left(seats: u32, booked: u32) -> u32 {
    seats.saturating_sub(booked)
}

fn total_price(unit_price: u32, quantity: u32) -> Option<u32> {
    unit_price.checked_mul(quantity)
}

fn next_frame(frame: u8) -> u8 {
    frame.wrapping_add(1)
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
