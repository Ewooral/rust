//! Rule 3: when the owner goes out of scope, the value is dropped.
//! Run: cargo run -p ownership --example scope_and_drop

struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  dropping {}", self.0);
    }
}

fn main() {
    println!("start of main");
    let _a = Noisy("a");
    {
        println!("entering inner scope");
        let _b = Noisy("b");
        println!("leaving inner scope");
    } // `_b` goes out of scope here, so it is dropped first.
    println!("end of main");
} // `_a` is dropped here.
