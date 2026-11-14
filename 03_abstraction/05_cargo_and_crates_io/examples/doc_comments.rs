//! Book 14.2: documentation comments, their common sections, and links.
//!
//! Run:       cargo run -p cargo_and_crates_io --example doc_comments
//! Read docs: cargo doc -p cargo_and_crates_io --example doc_comments --no-deps --open
//!
//! `///` documents the item BELOW it. `//!` documents the item it is INSIDE,
//! like this paragraph, which documents the whole example. Both are Markdown.
//! Doc tests (the code blocks) only run for a LIBRARY, so this lesson's real,
//! tested ones are in src/: run `cargo test -p cargo_and_crates_io --doc`.

/// Pocket money, counted in whole pence so there is no rounding.
pub mod piggy_bank {
    use std::fmt;

    /// A piggy bank. Make one with [`PiggyBank::new`], fill it with
    /// [`deposit`](PiggyBank::deposit), and empty it with
    /// [`PiggyBank::withdraw`].
    ///
    /// The square brackets are **intra-doc links**: rustdoc turns them into
    /// links and warns you if the item they name does not exist.
    #[derive(Debug, Default)]
    pub struct PiggyBank {
        pence: u64,
    }

    /// The error from [`PiggyBank::withdraw`]: not enough money.
    #[derive(Debug, PartialEq, Eq)]
    pub struct NotEnough {
        /// How much is in the bank.
        pub available: u64,
    }

    impl fmt::Display for NotEnough {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "only {}p in the bank", self.available)
        }
    }

    impl PiggyBank {
        /// An empty piggy bank.
        ///
        /// # Examples
        ///
        /// ```
        /// let bank = PiggyBank::new();
        /// assert_eq!(bank.balance(), 0);
        /// ```
        pub fn new() -> PiggyBank {
            PiggyBank::default()
        }

        /// How many pence are inside.
        pub fn balance(&self) -> u64 {
            self.pence
        }

        /// Puts money in.
        ///
        /// # Panics
        ///
        /// Panics if the bank would hold more than [`u64::MAX`] pence.
        pub fn deposit(&mut self, pence: u64) {
            self.pence = self.pence.checked_add(pence).expect("piggy bank overflow");
        }

        /// Takes money out and returns it.
        ///
        /// # Errors
        ///
        /// Returns [`NotEnough`] if the bank holds less than `pence`. The
        /// bank is unchanged.
        pub fn withdraw(&mut self, pence: u64) -> Result<u64, NotEnough> {
            if pence > self.pence {
                return Err(NotEnough {
                    available: self.pence,
                });
            }
            self.pence -= pence;
            Ok(pence)
        }

        /// Takes money out without checking there is enough.
        ///
        /// # Safety
        ///
        /// The bank must hold at least `pence`. Otherwise this is undefined
        /// behaviour, because it uses [`u64::unchecked_sub`], which skips
        /// the check. (Unsafe Rust is Book chapter 20.)
        pub unsafe fn withdraw_unchecked(&mut self, pence: u64) {
            // SAFETY: the caller promised that self.pence >= pence.
            self.pence = unsafe { self.pence.unchecked_sub(pence) };
        }
    }
}

use piggy_bank::PiggyBank;

fn main() {
    let mut bank = PiggyBank::new();
    bank.deposit(250);
    println!("balance: {}p", bank.balance());

    match bank.withdraw(1_000) {
        Ok(taken) => println!("took {taken}p"),
        Err(e) => println!("withdraw(1000) failed: {e}"), // the # Errors case
    }

    // SAFETY: the balance is 250, so taking 50 cannot go below zero.
    unsafe { bank.withdraw_unchecked(50) };
    println!("after withdraw_unchecked(50): {}p", bank.balance());

    println!("\nNow read these docs as a web page:");
    println!("  cargo doc -p cargo_and_crates_io --example doc_comments --no-deps --open");
}
