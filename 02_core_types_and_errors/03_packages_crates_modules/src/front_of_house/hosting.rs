//! `crate::front_of_house::hosting`: greeting guests and seating them.

/// The queue of guests waiting for a table.
#[derive(Debug, Default)]
pub struct Waitlist {
    names: Vec<String>, // private: only this module can touch the queue directly
}

impl Waitlist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

pub fn add_to_waitlist(list: &mut Waitlist, name: &str) {
    list.names.push(name.to_string());
}

/// Seats the guest who has waited longest, if there is one.
pub fn seat_at_table(list: &mut Waitlist) -> Option<String> {
    if list.names.is_empty() {
        return None;
    }
    let guest = list.names.remove(0);
    Some(format!("{guest} seated at table {}", table_for(&guest)))
}

/// Private helper: only code in `hosting` (and its children) can call it.
fn table_for(name: &str) -> usize {
    name.len() % 10 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_come_first_seated() {
        let mut list = Waitlist::new();
        assert!(list.is_empty());
        add_to_waitlist(&mut list, "Ama");
        add_to_waitlist(&mut list, "Kofi");
        assert_eq!(
            seat_at_table(&mut list),
            Some("Ama seated at table 4".into())
        );
        assert_eq!(list.len(), 1);
        assert!(seat_at_table(&mut list).is_some());
        assert_eq!(seat_at_table(&mut list), None);
    }
}
