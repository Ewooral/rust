//! Exercise 5: design a tidy public API with `pub use`.
//!
//! Users of `shop` should write `shop::Item` and `shop::total`, without
//! knowing about the `inventory` and `billing` modules inside.
//! Add TWO `pub use` lines where the TODO is. Do not move any code.
//!
//! Test: cargo test -p packages_crates_modules --features exercises --bin ex05_re_export

mod shop {
    pub mod inventory {
        #[derive(Debug, Clone, PartialEq)]
        pub struct Item {
            pub name: String,
            pub price: u32,
        }

        impl Item {
            pub fn new(name: &str, price: u32) -> Self {
                Self {
                    name: name.to_string(),
                    price,
                }
            }
        }
    }

    pub mod billing {
        use super::inventory::Item;

        /// Sum of all prices, in pence.
        pub fn total(items: &[Item]) -> u32 {
            items.iter().map(|item| item.price).sum()
        }
    }

    // TODO: two `pub use` lines here.
}

use shop::{Item, total};

fn main() {
    let basket = [Item::new("bread", 120), Item::new("milk", 95)];
    println!("{basket:?}");
    println!("total: {}p", total(&basket));
}

#[cfg(test)]
mod tests {
    #[test]
    fn short_paths_work() {
        let basket = vec![
            crate::shop::Item::new("tea", 300),
            crate::shop::Item::new("sugar", 150),
        ];
        assert_eq!(crate::shop::total(&basket), 450);
        assert_eq!(crate::shop::total(&[]), 0);
    }

    #[test]
    fn long_paths_still_work() {
        let item = crate::shop::inventory::Item::new("rice", 500);
        assert_eq!(crate::shop::billing::total(&[item]), 500);
    }
}
