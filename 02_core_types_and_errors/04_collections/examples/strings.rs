//! `String`: growable, owned, UTF-8 encoded text.
//! Run: cargo run -p collections --example strings

fn main() {
    // Creating
    let mut s = String::new();
    let data = "initial contents"; // &str: a borrowed string slice
    let s1 = data.to_string(); // any Display type has to_string
    let s2 = String::from("initial contents"); // the same result
    println!("{s:?} {s1:?} {s2:?}");

    // Any valid UTF-8 is fine.
    for hello in [
        "Dobrý den",
        "Hello",
        "שלום",
        "नमस्ते",
        "こんにちは",
        "Akwaaba",
    ] {
        print!("{hello} | ");
    }
    println!();

    // Updating
    s.push_str("foo"); // takes &str: does not take ownership
    let bar = "bar";
    s.push_str(bar);
    println!("bar is still usable: {bar}");
    s.push('!'); // push adds one char
    println!("{s}");

    // `+` takes ownership of the LEFT side and borrows the right:
    //   fn add(self, s: &str) -> String
    let hello = String::from("Hello, ");
    let world = String::from("world!");
    let joined = hello + &world; // `hello` is moved; `&String` coerces to `&str`
    // println!("{hello}"); // E0382: borrow of moved value: `hello`
    println!("{joined} ({world} still usable)");

    // For more than two pieces, format! is clearer and takes no ownership.
    let (t1, t2, t3) = ("tic", "tac", "toe");
    let game = format!("{t1}-{t2}-{t3}");
    println!("{game} (t1 is still usable: {t1})");

    // No indexing: a String is a Vec<u8> of UTF-8 bytes, and one character
    // can be 1 to 4 bytes long.
    let hola = String::from("Hola");
    let zdravstvuyte = String::from("Здравствуйте");
    // let h = hola[0]; // E0277: the type `str` cannot be indexed by `{integer}`
    println!(
        "'Hola' is {} bytes, 'Здравствуйте' is {} bytes for 12 letters",
        hola.len(),
        zdravstvuyte.len()
    );

    // Slicing with a byte range works, but must land on character boundaries.
    let first_two_letters = &zdravstvuyte[0..4]; // each Cyrillic letter is 2 bytes
    println!("first two letters: {first_two_letters}");
    // let half = &zdravstvuyte[0..1]; // panics: byte index 1 is not a char boundary

    // Iterate the way you mean: characters or bytes.
    for c in "Зд".chars() {
        print!("{c} ");
    }
    println!();
    for b in "Зд".bytes() {
        print!("{b} ");
    }
    println!();

    // Letters people see ("grapheme clusters") can be several chars, e.g. Hindi
    // "नमस्ते" is 6 chars but 4 letters. The std library does not do graphemes;
    // the `unicode-segmentation` crate does.
    println!("'नमस्ते' has {} chars", "नमस्ते".chars().count());

    // Useful methods: contains, replace, split, trim, to_uppercase, ...
    let line = "  Rust is fun  ";
    println!(
        "{:?} {} {:?}",
        line.trim(),
        line.contains("fun"),
        line.split_whitespace().collect::<Vec<_>>()
    );
}
