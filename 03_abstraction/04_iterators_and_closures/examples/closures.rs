//! Book 13.1: closures, anonymous functions that capture their environment.
//! Run: cargo run -p iterators_and_closures --example closures

#[derive(Debug, PartialEq, Copy, Clone)]
enum ShirtColor {
    Red,
    Blue,
}

struct Inventory {
    shirts: Vec<ShirtColor>,
}

impl Inventory {
    /// A customer may name a favourite colour. If not, they get whichever
    /// colour we have most of.
    fn giveaway(&self, user_preference: Option<ShirtColor>) -> ShirtColor {
        // `|| self.most_stocked()` is a closure: it takes no arguments and
        // CAPTURES `self` from the surrounding method. unwrap_or_else calls
        // it only when the Option is None.
        user_preference.unwrap_or_else(|| self.most_stocked())
    }

    fn most_stocked(&self) -> ShirtColor {
        let red = self
            .shirts
            .iter()
            .filter(|c| **c == ShirtColor::Red)
            .count();
        let blue = self.shirts.len() - red;
        if red > blue {
            ShirtColor::Red
        } else {
            ShirtColor::Blue
        }
    }
}

fn add_one_v1(x: u32) -> u32 {
    x + 1
}

fn main() {
    let store = Inventory {
        shirts: vec![ShirtColor::Blue, ShirtColor::Red, ShirtColor::Blue],
    };
    let user_pref1 = Some(ShirtColor::Red);
    println!("{user_pref1:?} gets {:?}", store.giveaway(user_pref1));
    let user_pref2 = None;
    println!("{user_pref2:?} gets {:?}", store.giveaway(user_pref2));

    // ---- Syntax: from a full function to the shortest closure ----
    let add_one_v2 = |x: u32| -> u32 { x + 1 }; // fully annotated
    let add_one_v3 = |x| x + 1; // types inferred from use below
    let add_one_v4 = |x| x + 1; // a body that is one expression needs no braces
    println!(
        "{} {} {} {}",
        add_one_v1(1),
        add_one_v2(1),
        add_one_v3(1u32),
        add_one_v4(1u32)
    );

    // Types are inferred from the FIRST use, then fixed:
    let example_closure = |x| x;
    let s = example_closure(String::from("hello"));
    // let n = example_closure(5); // E0308: expected `String`, found integer
    println!("{s}");

    // Unlike a fn, a closure can use variables from where it is defined.
    let greeting = String::from("Akwaaba");
    let greet = |name: &str| format!("{greeting}, {name}!");
    println!("{}", greet("Ama"));
    // fn greet_fn(name: &str) -> String { format!("{greeting}, {name}") }
    // ^ E0434: can't capture dynamic environment in a fn item
}
