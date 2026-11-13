//! Mini project for Book chapter 6: a parcel tracker.
//!
//! A parcel is always in exactly one state, and each state carries only the
//! data that makes sense for it. Events move it from one state to the next.
//! Impossible combinations, like "delivered but no signature field", or
//! "lost but has an ETA", cannot even be written down.
//!
//! Run: cargo run -p enums_and_pattern_matching
//! Test: cargo test -p enums_and_pattern_matching

#[derive(Debug, Clone, PartialEq)]
enum Status {
    Ordered,
    Shipped { tracking: String },
    OutForDelivery { tracking: String, eta_minutes: u32 },
    Delivered { signed_by: Option<String> },
    Lost,
}

#[derive(Debug)]
enum Event {
    Ship(String),
    LoadOnVan(u32),
    Deliver(Option<String>),
    ReportMissing,
}

impl Status {
    /// Applies an event. An event that makes no sense in the current state
    /// leaves the state unchanged.
    ///
    /// Matching on the pair `(self, event)` lets one `match` cover every
    /// combination, and the compiler checks we did not forget one.
    fn apply(self, event: Event) -> Status {
        match (self, event) {
            (Status::Ordered, Event::Ship(tracking)) => Status::Shipped { tracking },
            (Status::Shipped { tracking }, Event::LoadOnVan(eta_minutes)) => {
                Status::OutForDelivery {
                    tracking,
                    eta_minutes,
                }
            }
            (Status::OutForDelivery { .. }, Event::Deliver(signed_by)) => {
                Status::Delivered { signed_by }
            }
            // A parcel can go missing at any point before it is delivered.
            (Status::Delivered { signed_by }, Event::ReportMissing) => {
                Status::Delivered { signed_by }
            }
            (_, Event::ReportMissing) => Status::Lost,
            // Every other pair is not allowed: keep the state as it was.
            (unchanged, _) => unchanged,
        }
    }

    fn describe(&self) -> String {
        match self {
            Status::Ordered => String::from("Order received"),
            Status::Shipped { tracking } => format!("Shipped, tracking number {tracking}"),
            Status::OutForDelivery { eta_minutes, .. } => {
                format!("Out for delivery, arriving in about {eta_minutes} minutes")
            }
            Status::Delivered {
                signed_by: Some(name),
            } => format!("Delivered, signed for by {name}"),
            Status::Delivered { signed_by: None } => {
                String::from("Delivered, left in a safe place")
            }
            Status::Lost => String::from("Lost. We are sorry!"),
        }
    }

    /// The tracking number, if the parcel has one yet.
    fn tracking(&self) -> Option<&str> {
        match self {
            Status::Shipped { tracking } | Status::OutForDelivery { tracking, .. } => {
                Some(tracking)
            }
            _ => None,
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self, Status::Delivered { .. } | Status::Lost)
    }
}

fn main() {
    let events = vec![
        Event::Deliver(None), // too early: ignored
        Event::Ship(String::from("GH-0042")),
        Event::LoadOnVan(25),
        Event::Deliver(Some(String::from("Ama"))),
    ];

    let mut status = Status::Ordered;
    println!("{}", status.describe());
    for event in events {
        println!("  event: {event:?}");
        status = status.apply(event);
        println!("{}", status.describe());
        if let Some(t) = status.tracking() {
            println!("  (track it online with {t})");
        }
    }
    println!("finished? {}", status.is_finished());

    let unlucky = Status::Ordered
        .apply(Event::Ship(String::from("GH-0043")))
        .apply(Event::ReportMissing);
    println!("parcel GH-0043: {}", unlucky.describe());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delivered_parcel() -> Status {
        Status::Ordered
            .apply(Event::Ship("T1".into()))
            .apply(Event::LoadOnVan(10))
            .apply(Event::Deliver(None))
    }

    #[test]
    fn happy_path() {
        assert_eq!(delivered_parcel(), Status::Delivered { signed_by: None });
        assert!(delivered_parcel().is_finished());
    }

    #[test]
    fn out_of_order_events_are_ignored() {
        let s = Status::Ordered.apply(Event::LoadOnVan(5));
        assert_eq!(s, Status::Ordered);
        let s = Status::Ordered.apply(Event::Deliver(None));
        assert_eq!(s, Status::Ordered);
    }

    #[test]
    fn tracking_only_while_travelling() {
        assert_eq!(Status::Ordered.tracking(), None);
        let shipped = Status::Ordered.apply(Event::Ship("T9".into()));
        assert_eq!(shipped.tracking(), Some("T9"));
        assert_eq!(delivered_parcel().tracking(), None);
    }

    #[test]
    fn can_be_lost_before_delivery_but_not_after() {
        let lost = Status::Ordered
            .apply(Event::Ship("T2".into()))
            .apply(Event::ReportMissing);
        assert_eq!(lost, Status::Lost);
        assert!(lost.is_finished());
        assert_eq!(
            delivered_parcel().apply(Event::ReportMissing),
            delivered_parcel()
        );
    }

    #[test]
    fn describes_signature() {
        let s = Status::Delivered {
            signed_by: Some("Kofi".into()),
        };
        assert_eq!(s.describe(), "Delivered, signed for by Kofi");
    }
}
