//! Exercise 4: no loops allowed. Use iterator adapters only.
//!
//! Each function has a one-line description. Write each body as ONE
//! iterator chain (no `for`, no `while`, no `loop`, no `let mut`).
//! Useful: filter, map, sum, count, max_by_key, collect, any, all.
//!
//! Test: cargo test -p iterators_and_closures --features exercises --bin ex04_iterator_pipelines

#![allow(unused_variables, dead_code)]

#[derive(Debug, Clone, PartialEq)]
struct Product {
    name: String,
    price: u32, // pence
    stock: u32,
}

/// Total value of everything in stock: sum of price * stock.
fn stock_value(products: &[Product]) -> u32 {
    todo!()
}

/// Names of products that are sold out, in the original order.
fn sold_out(products: &[Product]) -> Vec<&str> {
    todo!()
}

/// The most expensive product that is in stock, if any.
fn priciest_available(products: &[Product]) -> Option<&Product> {
    todo!()
}

/// True if every product costs less than `limit`.
fn all_cheaper_than(products: &[Product], limit: u32) -> bool {
    todo!()
}

fn shop() -> Vec<Product> {
    vec![
        Product {
            name: "bread".into(),
            price: 120,
            stock: 10,
        },
        Product {
            name: "milk".into(),
            price: 95,
            stock: 0,
        },
        Product {
            name: "cheese".into(),
            price: 450,
            stock: 2,
        },
        Product {
            name: "caviar".into(),
            price: 9900,
            stock: 0,
        },
    ]
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_of_stock() {
        assert_eq!(stock_value(&shop()), 120 * 10 + 450 * 2);
        assert_eq!(stock_value(&[]), 0);
    }

    #[test]
    fn sold_out_names() {
        assert_eq!(sold_out(&shop()), ["milk", "caviar"]);
    }

    #[test]
    fn priciest_in_stock_skips_sold_out() {
        let products = shop();
        assert_eq!(
            priciest_available(&products).map(|p| p.name.as_str()),
            Some("cheese")
        );
        assert_eq!(priciest_available(&[]), None);
    }

    #[test]
    fn price_limit() {
        assert!(all_cheaper_than(&shop(), 10_000));
        assert!(!all_cheaper_than(&shop(), 500));
    }
}
