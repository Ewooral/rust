//! Book 18.1: objects contain data and behaviour, and encapsulation hides the
//! details. Rust has no `class`, but a struct + `impl` + privacy does the job.
//! Run: cargo run -p trait_objects_and_oop --example encapsulation
//!
//! `AveragedCollection` keeps a list of numbers AND a cached average. The two
//! must always agree, so both fields are private: the only way to change the
//! list is through `add` and `remove`, which update the average every time.

mod stats {
    /// Data (the fields) and behaviour (the methods) together: an "object".
    pub struct AveragedCollection {
        list: Vec<i32>, // private: nobody outside `stats` can touch it
        average: f64,   // private: nobody can set a wrong average
    }

    impl AveragedCollection {
        pub fn new() -> Self {
            AveragedCollection {
                list: Vec::new(),
                average: 0.0,
            }
        }

        pub fn add(&mut self, value: i32) {
            self.list.push(value);
            self.update_average();
        }

        pub fn remove(&mut self) -> Option<i32> {
            let result = self.list.pop();
            if result.is_some() {
                self.update_average();
            }
            result
        }

        /// Read-only access: callers may LOOK at the average, never set it.
        pub fn average(&self) -> f64 {
            self.average
        }

        /// A private helper: an implementation detail we may change freely.
        fn update_average(&mut self) {
            let total: i32 = self.list.iter().sum();
            self.average = if self.list.is_empty() {
                0.0
            } else {
                f64::from(total) / self.list.len() as f64
            };
        }
    }
}

use stats::AveragedCollection;

fn main() {
    let mut scores = AveragedCollection::new();
    for score in [70, 80, 90] {
        scores.add(score);
        println!("added {score:>2}  -> average {:.1}", scores.average());
    }
    println!(
        "removed {:?} -> average {:.1}",
        scores.remove(),
        scores.average()
    );

    // Encapsulation in action. These lines would break the invariant, so the
    // compiler refuses them:
    // scores.average = 99.0;   // E0616: field `average` of struct `AveragedCollection` is private
    // let fake = AveragedCollection { list: vec![1], average: 50.0 };
    //                          // E0451: fields `list` and `average` ... are private

    // Because nobody outside `stats` can see `list`, the module could swap the
    // Vec for a running total + count tomorrow, and this `main` would not
    // need a single change. That freedom is the point of encapsulation.
}
