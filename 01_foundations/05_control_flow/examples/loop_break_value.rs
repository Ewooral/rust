//! `loop` repeats until you `break`, and `break` can carry a value out.
//! Run: cargo run -p control_flow --example loop_break_value

fn main() {
    // The Book's example: keep doubling a counter, and stop at 10.
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2; // leave the loop, and make 20 its value
        }
    };
    println!("The result is {result}");

    // A typical use: retry something until it works. Here we "roll" numbers
    // from a fixed list until we get a six, skipping odd rolls with `continue`.
    let rolls: [u32; 6] = [3, 4, 1, 2, 6, 5];
    let mut attempt = 0;
    let tries_needed = loop {
        let roll = rolls[attempt];
        attempt += 1;
        if !roll.is_multiple_of(2) {
            println!("  rolled {roll}: odd, skip the rest of this turn");
            continue;
        }
        println!("  rolled {roll}");
        if roll == 6 {
            break attempt;
        }
    };
    println!("Got a six after {tries_needed} rolls");

    // `return` would leave the whole FUNCTION; `break` only leaves the loop.
    println!("still in main");
}
