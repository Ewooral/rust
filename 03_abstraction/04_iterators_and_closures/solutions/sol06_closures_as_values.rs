//! Solution to exercise 6: take and return closures.
//!
//! - `make_multiplier(n)` returns a closure that multiplies its input by n.
//! - `compose(f, g)` returns a closure that applies f, then g: g(f(x)).
//! - `apply_n_times(f, n, x)` applies f to x, n times in a row.
//! - `call_counter()` returns a closure that returns 1, 2, 3, ... each time
//!   it is called (it must remember its count: which Fn trait is that?).
//!
//! Fill in the bodies; the signatures are already right.
//!
//! Test: cargo test -p iterators_and_closures --features solutions --bin sol06_closures_as_values

fn make_multiplier(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n // move: the closure must own n, which dies when we return
}

fn compose<F, G>(f: F, g: G) -> impl Fn(i32) -> i32
where
    F: Fn(i32) -> i32,
    G: Fn(i32) -> i32,
{
    move |x| g(f(x))
}

fn apply_n_times(f: impl Fn(i32) -> i32, n: usize, x: i32) -> i32 {
    (0..n).fold(x, |acc, _| f(acc))
}

fn call_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    // FnMut: the closure changes the count it owns.
    move || {
        count += 1;
        count
    }
}

fn main() {
    let triple = make_multiplier(3);
    let add_then_triple = compose(|x| x + 1, triple);
    println!("(2 + 1) * 3 = {}", add_then_triple(2));
    println!("2^10 = {}", apply_n_times(|x| x * 2, 10, 1));
    let mut next = call_counter();
    println!("{} {} {}", next(), next(), next());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplier() {
        let triple = make_multiplier(3);
        assert_eq!(triple(5), 15);
        assert_eq!(triple(-2), -6);
    }

    #[test]
    fn composition_order_matters() {
        let add_one = |x| x + 1;
        let double = |x| x * 2;
        assert_eq!(compose(add_one, double)(5), 12); // (5 + 1) * 2
        assert_eq!(compose(double, add_one)(5), 11); // 5 * 2 + 1
    }

    #[test]
    fn repeated_application() {
        assert_eq!(apply_n_times(|x| x * 2, 3, 1), 8);
        assert_eq!(apply_n_times(|x| x + 1, 0, 7), 7);
    }

    #[test]
    fn counter_remembers() {
        let mut next = call_counter();
        assert_eq!((next(), next(), next()), (1, 2, 3));
        let mut fresh = call_counter();
        assert_eq!(fresh(), 1, "each counter has its own count");
    }
}
