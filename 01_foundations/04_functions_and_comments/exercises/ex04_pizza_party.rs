//! Exercise 4: small functions that work together.
//!
//! Implement the four functions. Each one is a single expression (no
//! `return`, no semicolon on the last line).
//!
//! - `slices_needed`: everyone eats `slices_each` slices.
//! - `pizzas_needed`: a pizza has `SLICES_PER_PIZZA` slices, and we can only
//!   buy whole pizzas, so round UP: 9 slices need 2 pizzas. Hint: for
//!   integers, `a.div_ceil(b)` is division that rounds up.
//! - `party_cost`: pizzas times the price of one, in pence.
//! - `party_plan`: use the three functions above and return
//!   `(pizzas, cost)` as a tuple.
//!
//! Test: cargo test -p functions_and_comments --features exercises --bin ex04_pizza_party

#![allow(unused_variables)]

const SLICES_PER_PIZZA: u32 = 8;

fn slices_needed(people: u32, slices_each: u32) -> u32 {
    todo!("slices for {people} people eating {slices_each} each")
}

fn pizzas_needed(slices: u32) -> u32 {
    todo!("whole pizzas for {slices} slices")
}

fn party_cost(pizzas: u32, price_pence: u32) -> u32 {
    todo!("{pizzas} pizzas at {price_pence}p")
}

fn party_plan(people: u32, slices_each: u32, price_pence: u32) -> (u32, u32) {
    todo!("(pizzas, cost) for {people} people")
}

fn main() {
    let (pizzas, cost) = party_plan(10, 3, 899);
    println!("Order {pizzas} pizzas for {cost}p");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_slices() {
        assert_eq!(slices_needed(10, 3), 30);
        assert_eq!(slices_needed(0, 3), 0);
    }

    #[test]
    fn rounds_pizzas_up() {
        assert_eq!(pizzas_needed(8), 1);
        assert_eq!(pizzas_needed(9), 2);
        assert_eq!(pizzas_needed(30), 4);
        assert_eq!(pizzas_needed(0), 0);
    }

    #[test]
    fn costs_the_order() {
        assert_eq!(party_cost(4, 899), 3596);
    }

    #[test]
    fn plans_the_party() {
        assert_eq!(party_plan(10, 3, 899), (4, 3596));
        assert_eq!(party_plan(3, 3, 1000), (2, 2000));
    }
}
