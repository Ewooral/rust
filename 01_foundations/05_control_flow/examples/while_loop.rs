//! `while` repeats as long as a condition stays true.
//! Run: cargo run -p control_flow --example while_loop

fn main() {
    // The Book's countdown.
    let mut number = 3;
    while number != 0 {
        println!("{number}!");
        number -= 1;
    }
    println!("LIFTOFF!!!");

    // `while` with an index works, but it's fragile: get the condition wrong
    // (`index < 6`) and the program panics, and the bounds check runs on
    // every pass.
    let a = [10, 20, 30, 40, 50];
    let mut index = 0;
    while index < a.len() {
        println!("the value is: {}", a[index]);
        index += 1;
    }

    // `while` shines when you don't know how many rounds there will be:
    // how many times can we halve 1000 before it drops below 1?
    let mut amount = 1000.0;
    let mut halvings = 0;
    while amount >= 1.0 {
        amount /= 2.0;
        halvings += 1;
    }
    println!("halved {halvings} times, ending at {amount}");
}
