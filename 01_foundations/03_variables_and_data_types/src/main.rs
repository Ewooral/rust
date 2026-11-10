fn main() {
    println!("...............");
    println!("VARIABLES AND DATA TYPES");
    // Data types in Rust - SCALER TYPES
    /*
     */
    let signed_int: i32 = 23;
    println!("signedInt: {}", signed_int);
    let unsigned_int: u32 = 45;
    println!("usigned_int: {}", unsigned_int);
    let float: f32 = 4.323;
    println!("float: {}", float);
    let double: f64 = 4.323;
    println!("double: {}", double);
    // suffix type annotation
    let an_integer = 5u64;
    println!("an_integer: {}", an_integer + 22);

    // COMPOUND TYPES
    // Tuple
    println!("...............");
    println!("COMPOUND TYPES::: ");
    let a_tuple: (i32, f64, bool, &str) = (1, 2.3, false, "hello");
    println!("a_tuple: {:?}", a_tuple);

    // Destructuring a tuple
    let (x, y, z, w) = a_tuple;
    println!("x: {}, y: {}, z: {}, w: {}", x, y, z, w);

    // Accessing a tuple element by index
    println!(
        "a_tuple.0: {}, a_tuple.1: {}, a_tuple.2: {}, a_tuple.3: {}",
        a_tuple.0, a_tuple.1, a_tuple.2, a_tuple.3
    );

    // Array
    let an_array: [i32; 5] = [1, 2, 3, 4, 5];
    println!("an_array: {:?}", an_array);

    // Integer addition
    println!("1 + 2 = {}", 1u32 + 2);

    // Integer subtraction
    println!("1 - 2 = {}", 1i32 - 2);
    // TODO ^ Try changing `1i32` to `1u32` to see why the type is important

    // Scientific notation
    println!("1e4 is {}, -2.5e-3 is {}", 1e4, -2.5e-3);

    // Short-circuiting boolean logic
    println!("true AND false is {}", true && false);
    println!("true OR false is {}", true || false);
    println!("NOT true is {}", !true);

    // Bitwise operations
    println!("0011 AND 0101 is {:04b}", 0b0011u32 & 0b0101);
    println!("0011 OR 0101 is {:04b}", 0b0011u32 | 0b0101);
    println!("0011 XOR 0101 is {:04b}", 0b0011u32 ^ 0b0101);
    println!("1 << 5 is {}", 1u32 << 5);
    println!("0x80 >> 2 is 0x{:x}", 0x80u32 >> 2);

    // Use underscores to improve readability!
    println!("One million is written as {}", 1_000_000u32);
}
