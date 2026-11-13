//! Exercise 2: make it compile (E0451), WITHOUT making `price` public.
//!
//! The cinema sets prices, not the customer: seats 1 to 10 cost 1200p, every
//! other seat costs 900p. Add a public constructor `Ticket::new(seat)` inside
//! the module that applies that rule, and use it in `main`.
//!
//! Run: cargo run -p packages_crates_modules --features exercises --bin ex02_fix_private_field

mod cinema {
    #[derive(Debug)]
    pub struct Ticket {
        pub seat: u32,
        price: u32,
    }

    impl Ticket {
        pub fn price(&self) -> u32 {
            self.price
        }
    }
}

fn main() {
    let front = cinema::Ticket { seat: 3, price: 0 };
    let back = cinema::Ticket { seat: 25, price: 0 };
    for t in [front, back] {
        println!("seat {} costs {}p", t.seat, t.price());
    }
}
