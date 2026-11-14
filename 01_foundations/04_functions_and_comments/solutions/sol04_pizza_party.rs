//! Solution to exercise 4: small functions that work together.
//!
//! Test: cargo test -p functions_and_comments --features solutions --bin sol04_pizza_party

const SLICES_PER_PIZZA: u32 = 8;

fn slices_needed(people: u32, slices_each: u32) -> u32 {
    people * slices_each
}

fn pizzas_needed(slices: u32) -> u32 {
    slices.div_ceil(SLICES_PER_PIZZA)
}

fn party_cost(pizzas: u32, price_pence: u32) -> u32 {
    pizzas * price_pence
}

fn party_plan(people: u32, slices_each: u32, price_pence: u32) -> (u32, u32) {
    let pizzas = pizzas_needed(slices_needed(people, slices_each));
    (pizzas, party_cost(pizzas, price_pence))
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
