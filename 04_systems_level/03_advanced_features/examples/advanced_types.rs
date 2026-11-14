//! Book 20.3: advanced types.
//! Run: cargo run -p advanced_features --example advanced_types
//!
//! 1. newtypes for safety and hiding   2. type aliases
//! 3. the never type `!`               4. dynamically sized types and `Sized`

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{self, Debug};
use std::io;

// ---- 1. Newtypes: a new type that the compiler keeps apart ----------------------
#[derive(Debug, Clone, Copy)]
struct UserId(u32);
#[derive(Debug, Clone, Copy)]
struct OrderId(u32);

fn cancel(order: OrderId) {
    println!("cancelled order {}", order.0);
}

// A newtype can also HIDE its inside: callers of People see only `add` and
// `name_of`, not the HashMap, so the representation can change later.
struct People {
    names: HashMap<u32, String>,
    next_id: u32,
}

impl People {
    fn new() -> People {
        People {
            names: HashMap::new(),
            next_id: 1,
        }
    }
    fn add(&mut self, name: &str) -> UserId {
        let id = self.next_id;
        self.next_id += 1;
        self.names.insert(id, name.to_string());
        UserId(id)
    }
    fn name_of(&self, id: UserId) -> Option<&str> {
        self.names.get(&id.0).map(String::as_str)
    }
}

// ---- 2. Type aliases: a new NAME, not a new type ---------------------------------
type Kilometers = i32;

// Long types get short names. Thunk = "a boxed job to run later".
type Thunk = Box<dyn Fn() + Send + 'static>;

fn returns_long_type() -> Thunk {
    Box::new(|| println!("hi from a Thunk"))
}

// std does this too: io::Result<T> is Result<T, io::Error>.
fn read_config() -> io::Result<String> {
    Ok(String::from("threads = 4"))
}

// ---- 3. The never type: `!` -------------------------------------------------------
// A function returning `!` never returns: it loops forever, panics or exits.
fn fail(message: &str) -> ! {
    panic!("{message}");
}

fn parse_guess(text: &str) -> u32 {
    // match arms must have one type. `continue`, `return`, `panic!` and
    // `fail()` all have type `!`, which COERCES into any type, so this works.
    match text.trim().parse() {
        Ok(n) => n,
        Err(_) if text.is_empty() => 0,
        Err(_) => fail("not a number"),
    }
}

// Stable Rust doesn't let you write `Result<T, !>` yet. std::convert::Infallible
// is an empty enum that plays the same role: an error that can't happen.
fn always_works() -> Result<u8, Infallible> {
    Ok(42)
}

// ---- 4. Dynamically sized types (DSTs) and Sized --------------------------------
// `str`, `[T]` and `dyn Trait` have no size known at compile time, so they
// always live behind a pointer (&str, Box<[T]>, &dyn Trait), which then
// carries the size or a vtable along with the address.
//
// Every generic gets an invisible `T: Sized` bound. `?Sized` removes it:
fn describe<T: Debug + ?Sized>(value: &T) -> String {
    format!("{value:?}: size_of_val = {}", size_of_val(value))
}
// Without `?Sized`, describe("hi") is E0277: the size for values of type `str` cannot be known at compilation time

struct Celsius(f64);
impl fmt::Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°C", self.0)
    }
}

fn main() {
    println!("== 1. newtypes ==");
    let order = OrderId(7);
    let user = UserId(7);
    cancel(order);
    // cancel(user); // E0308: mismatched types, expected `OrderId`, found `UserId`
    println!("user {user:?} is a different type, even with the same number inside");
    let mut people = People::new();
    let ama = people.add("Ama");
    println!("{:?} is {:?}", ama, people.name_of(ama));

    println!("\n== 2. type aliases ==");
    let x: i32 = 5;
    let y: Kilometers = 5;
    println!("x + y = {} (an alias mixes freely with i32)", x + y);
    returns_long_type()();
    println!("config: {:?}", read_config());

    println!("\n== 3. the never type ==");
    println!("parse_guess(\"42\") = {}", parse_guess("42"));
    println!("parse_guess(\"\") = {}", parse_guess(""));
    // parse_guess("forty") would panic: fail() never comes back.
    let Ok(answer) = always_works(); // no Err arm needed: Infallible has no values
    println!("an infallible result: {answer}");

    println!("\n== 4. DSTs and Sized ==");
    // let s: str = *"hello"; // E0277: the size for values of type `str` cannot be known at compilation time
    let s: &str = "hello";
    println!("&str is {} bytes: address + length", size_of::<&str>());
    println!("&u8  is {} bytes: just an address", size_of::<&u8>());
    println!(
        "&dyn Display is {} bytes: address + vtable",
        size_of::<&dyn fmt::Display>()
    );
    println!("{}", describe(&42u8));
    println!("{}", describe(s));
    println!("{}", describe(&[1, 2, 3][..]));
    let temp: Box<dyn fmt::Display> = Box::new(Celsius(21.5));
    println!("a Box<dyn Display>: {temp}");
}
