//! Exercise 1: make it compile (E0603).
//!
//! Open the right door with ONE word. Leave `boil_water` private: customers
//! should be able to order tea, not operate the kettle.
//!
//! Run: cargo run -p packages_crates_modules --features exercises --bin ex01_fix_private_function

mod cafe {
    pub mod kitchen {
        fn make_tea() -> String {
            format!("tea made with {}", boil_water())
        }

        fn boil_water() -> &'static str {
            "freshly boiled water"
        }
    }
}

fn main() {
    println!("{}", cafe::kitchen::make_tea());
}
