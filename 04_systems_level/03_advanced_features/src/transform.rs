//! Moving points around with FUNCTION POINTERS and CLOSURES (Book 20.4).

use crate::geometry::Vec2;

/// A TYPE ALIAS (Book 20.3) for a function pointer type. It is just another
/// name for `fn(Vec2) -> Vec2`, not a new type, but it is shorter to read.
pub type Transform = fn(Vec2) -> Vec2;

/// Doubles a vector. A plain function: it can go in the table below.
fn double(v: Vec2) -> Vec2 {
    v * 2.0
}

/// Named transforms. Because a `fn` pointer is a plain value (one address),
/// a table of them can even be a `const`. Methods and free functions mix.
pub const NAMED: [(&str, Transform); 4] = [
    ("flip_x", Vec2::flip_x),
    ("flip_y", Vec2::flip_y),
    ("rotate90", Vec2::rotate90),
    ("double", double),
];

/// Finds a transform by name, e.g. from a command or a config file.
pub fn lookup(name: &str) -> Option<Transform> {
    NAMED
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|&(_, transform)| transform)
}

/// RETURNING A CLOSURE: the closure captures `by`, so it can't be a plain
/// `fn` pointer. `impl Fn` names "some closure type" without spelling it out.
pub fn translate(by: Vec2) -> impl Fn(Vec2) -> Vec2 {
    move |v| v + by
}

/// Another returned closure. `translate` and `scale` return DIFFERENT types,
/// even though both are written `impl Fn(Vec2) -> Vec2`.
pub fn scale(factor: f64) -> impl Fn(Vec2) -> Vec2 {
    move |v| v * factor
}

/// One closure that runs several steps in order. The steps have different
/// types, so they are boxed as trait objects to share one `Vec`.
pub fn chain(steps: Vec<Box<dyn Fn(Vec2) -> Vec2>>) -> Box<dyn Fn(Vec2) -> Vec2> {
    Box::new(move |v| steps.iter().fold(v, |acc, step| step(acc)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_named_function_pointers() {
        let rotate = lookup("rotate90").expect("rotate90 is in the table");
        assert_eq!(rotate(Vec2::new(1.0, 0.0)), Vec2::new(0.0, 1.0));
        assert_eq!(
            lookup("double").map(|f| f(Vec2::new(1.0, 2.0))),
            Some(Vec2::new(2.0, 4.0))
        );
        assert!(lookup("teleport").is_none());
    }

    #[test]
    fn returned_closures_remember_their_settings() {
        let nudge = translate(Vec2::new(1.0, -1.0));
        let grow = scale(3.0);
        assert_eq!(nudge(Vec2::ZERO), Vec2::new(1.0, -1.0));
        assert_eq!(grow(Vec2::new(1.0, 2.0)), Vec2::new(3.0, 6.0));
    }

    #[test]
    fn chain_runs_steps_in_order() {
        let steps: Vec<Box<dyn Fn(Vec2) -> Vec2>> = vec![
            Box::new(scale(2.0)),
            Box::new(translate(Vec2::new(1.0, 0.0))),
            Box::new(Vec2::rotate90), // a fn pointer fits in the same box
        ];
        let recipe = chain(steps);
        // (1, 1) -> (2, 2) -> (3, 2) -> (-2, 3)
        assert_eq!(recipe(Vec2::new(1.0, 1.0)), Vec2::new(-2.0, 3.0));
    }
}
