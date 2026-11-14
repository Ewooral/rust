//! Mini project for Book 3.3-3.4: a shop receipt built from small functions.
//!
//! Every function does one job, takes what it needs as parameters, and hands
//! back one value. Prices are whole **pence** in a `u32`, so there's no
//! floating-point rounding to worry about: `1999` means £19.99.
//!
//! These `//!` lines document the whole crate, and the `///` lines document
//! the item below them. Turn them into a website with:
//!
//! ```text
//! cargo doc -p functions_and_comments --open
//! ```
//!
//! Run: cargo run -p functions_and_comments
//! Test: cargo test -p functions_and_comments

/// VAT (sales tax) in percent, added after any discount.
const VAT_PERCENT: u32 = 20;

fn main() {
    // Each item is (name, price in pence, quantity).
    let basket = [
        ("Notebook", 250, 3),
        ("Pen", 120, 5),
        ("Rust book", 3999, 1),
    ];
    let discount_percent = 10;

    println!("RECEIPT");
    let mut subtotal = 0;
    // `for` runs once per item in the basket (loops are lesson 05).
    for (item, price, quantity) in basket {
        let cost = line_total(price, quantity);
        subtotal += cost;
        println!(
            "{}",
            row(&format!("{quantity} x {item}"), &format_pounds(cost))
        );
    }

    // One function call, three answers: destructure the tuple it returns.
    let (discount, vat, total) = checkout(subtotal, discount_percent);

    println!("{}", "-".repeat(28));
    println!("{}", row("Subtotal", &format_pounds(subtotal)));
    println!(
        "{}",
        row(
            &format!("Discount {discount_percent}%"),
            &format!("-{}", format_pounds(discount))
        )
    );
    println!(
        "{}",
        row(
            &format!("VAT {VAT_PERCENT}%"),
            &format!("+{}", format_pounds(vat))
        )
    );
    println!("{}", row("TOTAL", &format_pounds(total)));
}

/// The cost of one line on the receipt: unit price times quantity, in pence.
fn line_total(unit_pence: u32, quantity: u32) -> u32 {
    unit_pence * quantity // no semicolon: this is the value we return
}

/// `percent` percent of `amount`, rounded to the nearest penny (halves go up).
///
/// For example, 10% of 1999p is 199.9p, which rounds to 200p.
fn percent_of(amount: u32, percent: u32) -> u32 {
    // Adding 50 before dividing by 100 rounds to the nearest whole penny.
    (amount * percent + 50) / 100
}

/// Applies the discount, then VAT, and returns `(discount, vat, total)`.
///
/// Returning a tuple is how a function hands back several values at once.
fn checkout(subtotal: u32, discount_percent: u32) -> (u32, u32, u32) {
    let discount = percent_of(subtotal, discount_percent);

    // A block `{ ... }` is an expression: its value is its last expression.
    let vat = {
        let after_discount = subtotal - discount;
        percent_of(after_discount, VAT_PERCENT)
    };

    (discount, vat, subtotal - discount + vat)
}

/// Formats pence as pounds: `1999` becomes `"£19.99"` and `5` becomes `"£0.05"`.
fn format_pounds(pence: u32) -> String {
    // `{:02}` pads with a zero to two digits, so 5p shows as 05.
    format!("£{}.{:02}", pence / 100, pence % 100)
}

/// One receipt row: the label on the left, the amount lined up on the right.
fn row(label: &str, amount: &str) -> String {
    format!("{label:<18}{amount:>10}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_total_multiplies_price_by_quantity() {
        assert_eq!(line_total(250, 3), 750);
        assert_eq!(line_total(3999, 1), 3999);
        assert_eq!(line_total(120, 0), 0);
    }

    #[test]
    fn percent_of_rounds_to_the_nearest_penny() {
        assert_eq!(percent_of(1000, 10), 100);
        assert_eq!(percent_of(1999, 10), 200); // 199.9 rounds up
        assert_eq!(percent_of(1994, 10), 199); // 199.4 rounds down
        assert_eq!(percent_of(5, 10), 1); // 0.5 rounds up
        assert_eq!(percent_of(0, 20), 0);
    }

    #[test]
    fn checkout_applies_discount_then_vat() {
        // 5349p - 10% (535p) = 4814p, + 20% VAT (963p) = 5777p
        assert_eq!(checkout(5349, 10), (535, 963, 5777));
        assert_eq!(checkout(10_000, 0), (0, 2_000, 12_000));
    }

    #[test]
    fn formats_pounds_and_pence() {
        assert_eq!(format_pounds(1999), "£19.99");
        assert_eq!(format_pounds(5), "£0.05");
        assert_eq!(format_pounds(100), "£1.00");
        assert_eq!(format_pounds(0), "£0.00");
    }

    #[test]
    fn rows_line_up() {
        let a = row("Pen", "£1.20");
        let b = row("Subtotal", "£53.49");
        assert_eq!(a.chars().count(), b.chars().count());
        assert!(a.ends_with("£1.20"));
    }
}
