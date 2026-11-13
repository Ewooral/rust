//! Beyond the Book: two patterns built from plain structs.
//! Run: cargo run -p structs --example newtype_and_builder

// ---- Newtype: wrap a primitive so the compiler stops you mixing units ----

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Metres(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Seconds(f64);

fn speed(distance: Metres, time: Seconds) -> f64 {
    distance.0 / time.0
}

// ---- Builder: many optional settings, set step by step ----

#[derive(Debug)]
struct Pizza {
    size_cm: u32,
    cheese: bool,
    toppings: Vec<String>,
}

#[derive(Debug, Default)]
struct PizzaBuilder {
    size_cm: Option<u32>,
    cheese: bool,
    toppings: Vec<String>,
}

impl PizzaBuilder {
    fn new() -> Self {
        Self::default()
    }

    // Each step takes `self` and returns it, so calls chain.
    fn size(mut self, cm: u32) -> Self {
        self.size_cm = Some(cm);
        self
    }

    fn cheese(mut self) -> Self {
        self.cheese = true;
        self
    }

    fn topping(mut self, t: &str) -> Self {
        self.toppings.push(t.to_string());
        self
    }

    fn build(self) -> Pizza {
        Pizza {
            size_cm: self.size_cm.unwrap_or(30),
            cheese: self.cheese,
            toppings: self.toppings,
        }
    }
}

fn main() {
    let d = Metres(100.0);
    let t = Seconds(9.58);
    println!("Bolt: {:.2} m/s", speed(d, t));
    // speed(t, d); // E0308: mismatched types, expected `Metres`, found `Seconds`

    let pizza = PizzaBuilder::new()
        .size(35)
        .cheese()
        .topping("mushroom")
        .topping("pepper")
        .build();
    println!("{pizza:?}");
    println!(
        "{} cm, cheese: {}, {} toppings",
        pizza.size_cm,
        pizza.cheese,
        pizza.toppings.len()
    );

    let plain = PizzaBuilder::new().build();
    println!("{plain:?}");
}
