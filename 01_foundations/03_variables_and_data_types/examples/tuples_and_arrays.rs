//! The two compound types: tuples (mixed types) and arrays (one type, fixed length).
//! Run: cargo run -p variables_and_data_types --example tuples_and_arrays
//!
//! Then ask for a month that doesn't exist and watch Rust stop the program:
//! cargo run -p variables_and_data_types --example tuples_and_arrays -- 12

fn main() {
    // A tuple groups values of DIFFERENT types. Its length never changes.
    let tup: (i32, f64, u8) = (500, 6.4, 1);

    // Destructuring: take it apart into separate variables.
    let (x, y, z) = tup;
    println!("x = {x}, y = {y}, z = {z}");

    // Or reach one element with `.` and its index, starting from 0.
    println!("tup.0 = {}, tup.1 = {}", tup.0, tup.1);

    // The empty tuple `()` is called "unit": the value of "nothing".
    let unit = ();
    println!("unit looks like {unit:?}");

    // An array holds values of the SAME type, and its length is part of its
    // type: [&str; 12] is "12 string slices".
    let months: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let zeros = [0; 5]; // five zeros: [value; length]
    println!(
        "{} months, first {}, last {}",
        months.len(),
        months[0],
        months[11]
    );
    println!("zeros = {zeros:?}");

    // Indexing is checked. With an index only known at run time (here, from
    // the command line), a bad one stops the program with a panic instead of
    // reading whatever memory happens to be next to the array.
    let arg = std::env::args().nth(1).unwrap_or(String::from("4"));
    let index: usize = arg.trim().parse().expect("pass a month index from 0 to 11");
    println!("months[{index}] = {}", months[index]);
}
