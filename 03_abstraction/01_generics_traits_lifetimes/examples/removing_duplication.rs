//! Book 10.0: before generics, remove duplication by extracting a function.
//! Run: cargo run -p generics_traits_lifetimes --example removing_duplication

/// Step 2: the repeated loop, extracted into a function. It works for ANY
/// list of i32, but only for i32. Generics (next example) remove that limit.
fn largest(list: &[i32]) -> &i32 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    // Step 1: the same loop written out twice (copy-paste duplication).
    let number_list = vec![34, 50, 25, 100, 65];
    let mut biggest = &number_list[0];
    for number in &number_list {
        if number > biggest {
            biggest = number;
        }
    }
    println!("The largest number is {biggest}");

    // Step 2: call the function instead, once per list.
    let number_list = vec![102, 34, 6000, 89, 54, 2, 43, 8];
    println!("The largest number is {}", largest(&number_list));
    println!("The largest number is {}", largest(&[34, 50, 25, 100, 65]));

    // The same three steps lead to generics:
    //   1. spot duplicate code,
    //   2. extract it into a function, naming what varies as parameters,
    //   3. call the function instead.
    // Generics do the same, except the thing that varies is a TYPE.
}
