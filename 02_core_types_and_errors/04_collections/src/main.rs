use collections::{arrays, strings};

fn main() {
    println!("{:?}", strings::group_in_pairs("abcdefghi"));
    println!("{:?}", arrays::index_of(&[1, 2, 3, 4], 3));
    println!("sum = {}", arrays::sum(&[1, 2, 3, 4]));
}
