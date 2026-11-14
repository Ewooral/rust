//! Turning text into a number with `parse`, and reusing a name by shadowing.
//! Run: cargo run -p guessing_game --example parse_and_shadow

fn main() {
    // What `read_line` gives us: text, with the newline still on the end.
    let guess = String::from("  42\n");

    // SHADOWING: a new `guess` that hides the old one. It can even have a
    // different type: the old `guess` was a String, the new one is a u32.
    // `trim` drops the spaces and newline; `parse` needs to know the target
    // type, which the `: u32` annotation tells it.
    let guess: u32 = guess.trim().parse().expect("Please type a number!");
    println!("guess + 1 = {}", guess + 1);

    // `parse` returns a Result. `match` lets us handle both outcomes instead
    // of crashing with `expect`. (`for` runs the body once per item; loops
    // are lesson 05.) `::<u32>` is another way to say which type we want.
    let inputs = ["7", "  100 ", "", "seven", "-5", "3.5", "99999999999"];
    for input in inputs {
        match input.trim().parse::<u32>() {
            Ok(num) => println!("{input:?} -> Ok({num})"),
            Err(e) => println!("{input:?} -> Err: {e}"),
        }
    }
}
