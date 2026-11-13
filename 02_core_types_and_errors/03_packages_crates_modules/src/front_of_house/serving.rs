//! `crate::front_of_house::serving`: taking orders and payment.

use crate::back_of_house::Appetizer; // absolute path from the crate root

/// Prices are in pence.
pub fn price_of(starter: &Appetizer) -> u32 {
    match starter {
        Appetizer::Soup => 450,
        Appetizer::Salad => 525,
    }
}

pub fn take_order(starters: &[Appetizer]) -> u32 {
    starters.iter().map(price_of).sum()
}

/// Seats the next guest and hands them a menu.
/// `super` is the parent module (`front_of_house`), so `super::hosting` is
/// our sibling module. (`super::super` would be the crate root.)
pub fn serve_next(list: &mut super::hosting::Waitlist) -> Option<String> {
    super::hosting::seat_at_table(list).map(|seated| format!("{seated}, menu handed over"))
}

/// `pub(super)`: visible to the parent module `front_of_house` and its
/// children, but not to the rest of the crate.
pub(super) fn tip(total: u32) -> u32 {
    total / 10
}

pub fn take_payment(total: u32) -> String {
    let with_tip = total + tip(total);
    format!("paid £{}.{:02}", with_tip / 100, with_tip % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_and_tips() {
        let total = take_order(&[Appetizer::Soup, Appetizer::Salad]);
        assert_eq!(total, 975);
        assert_eq!(tip(total), 97);
        assert_eq!(take_payment(total), "paid £10.72");
    }

    #[test]
    fn serve_next_uses_the_sibling_module() {
        let mut list = super::super::hosting::Waitlist::new();
        assert_eq!(serve_next(&mut list), None);
        super::super::hosting::add_to_waitlist(&mut list, "Esi");
        assert_eq!(
            serve_next(&mut list),
            Some("Esi seated at table 4, menu handed over".into())
        );
    }
}
