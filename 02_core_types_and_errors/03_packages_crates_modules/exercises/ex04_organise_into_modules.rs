//! Exercise 4: organise loose functions into modules.
//!
//! `main` and the tests already use the paths they want:
//!   math::geometry::rectangle_area, math::geometry::circle_area,
//!   math::stats::mean
//! Wrap the functions below in `mod math { pub mod geometry {..} pub mod stats {..} }`
//! so those paths work. Rules:
//!   - `sum` is a helper for `mean` only: keep it PRIVATE inside `stats`.
//!   - Do not change any function body.
//!
//! Test: cargo test -p packages_crates_modules --features exercises --bin ex04_organise_into_modules

use std::f64::consts::PI;

fn rectangle_area(width: f64, height: f64) -> f64 {
    width * height
}

fn circle_area(radius: f64) -> f64 {
    PI * radius * radius
}

fn sum(values: &[f64]) -> f64 {
    values.iter().sum()
}

/// The average, or None for an empty slice.
fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(sum(values) / values.len() as f64)
    }
}

fn main() {
    println!("rectangle: {}", math::geometry::rectangle_area(3.0, 4.0));
    println!("circle: {:.2}", math::geometry::circle_area(1.0));
    println!("mean: {:?}", math::stats::mean(&[1.0, 2.0, 6.0]));
}

#[cfg(test)]
mod tests {
    use super::math::{geometry, stats};

    #[test]
    fn geometry_works() {
        assert_eq!(geometry::rectangle_area(2.0, 5.0), 10.0);
        assert!((geometry::circle_area(2.0) - 4.0 * std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn stats_works() {
        assert_eq!(stats::mean(&[2.0, 4.0]), Some(3.0));
        assert_eq!(stats::mean(&[]), None);
    }
}
