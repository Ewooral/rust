use macros_in_depth::memoize;

#[memoize]
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn main() {}
