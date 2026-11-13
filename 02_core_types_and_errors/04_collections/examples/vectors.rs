//! `Vec<T>`: a growable list of values of one type, stored next to each other.
//! Run: cargo run -p collections --example vectors

#[derive(Debug)]
enum SpreadsheetCell {
    Int(i32),
    Float(f64),
    Text(String),
}

fn main() {
    // Creating
    let mut v: Vec<i32> = Vec::new(); // empty: needs a type annotation
    let w = vec![1, 2, 3]; // the vec! macro: type inferred as Vec<i32>
    println!("{v:?} {w:?}");

    // Updating: the vector must be `mut`
    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
    println!("after push: {v:?}, len {}", v.len());
    println!("pop gives {:?}, now {v:?}", v.pop());

    // Reading: indexing panics when out of range, `get` returns an Option.
    let third: &i32 = &v[2];
    println!("The third element is {third}");
    match v.get(2) {
        Some(third) => println!("get(2) found {third}"),
        None => println!("There is no third element."),
    }
    println!("get(100) = {:?}", v.get(100));
    // let does_not_exist = &v[100]; // panics: index out of bounds: the len is 3 but the index is 100

    // The borrowing rules apply. While `first` borrows an element, pushing
    // is forbidden: push may move the whole buffer to a bigger allocation.
    let first = &v[0];
    // v.push(6); // E0502: cannot borrow `v` as mutable because it is also borrowed as immutable
    println!("The first element is: {first}");
    v.push(9); // fine now: `first` is no longer used

    // Iterating: by shared reference, or by mutable reference.
    for i in &v {
        print!("{i} ");
    }
    println!();
    for i in &mut v {
        *i += 50; // `*` dereferences to change the value the reference points to
    }
    println!("after +50: {v:?}");

    // One type per Vec, but an enum lets one Vec hold "different" values.
    let row = vec![
        SpreadsheetCell::Int(3),
        SpreadsheetCell::Text(String::from("blue")),
        SpreadsheetCell::Float(10.12),
    ];
    for cell in &row {
        match cell {
            SpreadsheetCell::Int(n) => println!("int {n}"),
            SpreadsheetCell::Float(f) => println!("float {f}"),
            SpreadsheetCell::Text(t) => println!("text {t}"),
        }
    }

    // Dropping a Vec drops every element it owns.
    {
        let mut scoped = vec![String::from("a"), String::from("b")];
        scoped.push(String::from("c"));
        println!("{} strings about to be dropped", scoped.len());
    } // all three Strings and the Vec's buffer are freed here
}
