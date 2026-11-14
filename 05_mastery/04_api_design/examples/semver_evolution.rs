//! Evolving an API without breaking its users. Version 1 and version 2 of
//! a tiny library side by side; the comments say which changes would break
//! a user's code, and the error they'd see.
//!
//! Run: cargo run -p api_design --example semver_evolution

mod v1 {
    /// Public fields: users may build it with a struct literal.
    pub struct Config {
        pub port: u16,
    }

    /// No `#[non_exhaustive]`: users may match it exhaustively.
    pub enum Level {
        Low,
        High,
    }

    pub trait Store {
        fn get(&self, key: &str) -> Option<String>;
    }

    pub fn connect(host: &str) -> String {
        format!("connected to {host}")
    }
}

mod v2 {
    // BREAKING: adding a public field. `Config { port: 80 }` in a user's code:
    //     error[E0063]: missing field `host` in initializer of `Config`
    // Not breaking: a private field plus a constructor, or `#[non_exhaustive]`
    // from the start. We do the latter here.
    #[non_exhaustive]
    pub struct Config {
        pub port: u16,
        pub host: String,
    }

    impl Config {
        pub fn new(port: u16) -> Self {
            Config {
                port,
                host: "localhost".into(),
            }
        }
    }

    // BREAKING without `#[non_exhaustive]`: adding a variant. A user's
    //     match level { Level::Low => …, Level::High => … }
    //     error[E0004]: non-exhaustive patterns: `v2::Level::Medium` not covered
    pub enum Level {
        Low,
        Medium,
        High,
    }

    pub trait Store {
        fn get(&self, key: &str) -> Option<String>;
        // BREAKING: a new REQUIRED method. Every user's `impl Store`:
        //     error[E0046]: not all trait items implemented, missing: `contains`
        // Not breaking: give it a default body (or seal the trait).
        fn contains(&self, key: &str) -> bool {
            self.get(key).is_some()
        }
    }

    // BREAKING: a new parameter. `connect("db")`:
    //     error[E0061]: this function takes 2 arguments but 1 argument was supplied
    // Not breaking: a new function next to the old one, and deprecate the old.
    #[deprecated(since = "2.0.0", note = "use `connect_on` to choose the port")]
    pub fn connect(host: &str) -> String {
        connect_on(host, 5432)
    }

    pub fn connect_on(host: &str, port: u16) -> String {
        format!("connected to {host}:{port}")
    }
}

struct Memory;

impl v1::Store for Memory {
    fn get(&self, key: &str) -> Option<String> {
        (key == "answer").then(|| "42".into())
    }
}

// The same impl works unchanged against v2: `contains` has a default.
impl v2::Store for Memory {
    fn get(&self, key: &str) -> Option<String> {
        (key == "answer").then(|| "42".into())
    }
}

fn main() {
    let old = v1::Config { port: 80 };
    let new = v2::Config::new(80); // the only way to build it from another crate
    println!("v1 port {}; v2 {}:{}", old.port, new.host, new.port);

    for level in [v1::Level::Low, v1::Level::High] {
        let name = match level {
            v1::Level::Low => "low",
            v1::Level::High => "high",
        };
        print!("v1 {name}  ");
    }
    for level in [v2::Level::Low, v2::Level::Medium, v2::Level::High] {
        let name = match level {
            v2::Level::Low => "low",
            v2::Level::Medium => "medium",
            v2::Level::High => "high",
        };
        print!("v2 {name}  ");
    }
    println!();

    println!("v2 contains: {}", v2::Store::contains(&Memory, "answer"));
    println!("v1 get: {:?}", v1::Store::get(&Memory, "answer"));

    println!("{}", v1::connect("db"));
    #[allow(deprecated)] // without this: warning: use of deprecated function `v2::connect`
    let still_works = v2::connect("db");
    println!("{still_works} (deprecated, but still there)");
    println!("{}", v2::connect_on("db", 6543));
}
