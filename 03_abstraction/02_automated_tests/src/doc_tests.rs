//! Beyond the Book: documentation tests.
//!
//! Every ```rust code block in a doc comment (`///` or `//!`) is compiled
//! and run by `cargo test`, as if it were a tiny program using this crate.
//! So the examples in your docs can never silently go out of date.
//!
//! Run only the doc tests: `cargo test -p automated_tests --doc`

/// Adds two numbers.
///
/// A plain example. It passes if it compiles and runs without panicking:
///
/// ```
/// use automated_tests::doc_tests::add;
/// assert_eq!(add(2, 2), 4);
/// ```
///
/// Lines starting with `# ` are compiled but HIDDEN from the rendered docs,
/// so readers see only the interesting part:
///
/// ```
/// # use automated_tests::doc_tests::add;
/// let total = add(40, 2);
/// # assert_eq!(total, 42);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Divides, refusing to divide by zero.
///
/// # Panics
///
/// Panics if `b` is zero. A `should_panic` block passes only if it panics:
///
/// ```should_panic
/// automated_tests::doc_tests::divide(1, 0);
/// ```
pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Divide-by-zero error");
    }
    a / b
}

/// Shows the other doc-test attributes.
///
/// `no_run`: compiled, never run (for code that would touch the network,
/// loop forever, or delete files):
///
/// ```no_run
/// loop {
///     std::thread::sleep(std::time::Duration::from_secs(60));
/// }
/// ```
///
/// `compile_fail`: passes only if the code does NOT compile. Handy for
/// proving that a mistake is impossible:
///
/// ```compile_fail
/// let s = String::from("moved");
/// let t = s;
/// println!("{s}"); // E0382: borrow of moved value
/// ```
///
/// `text` (or any non-Rust language): not compiled at all:
///
/// ```text
/// just some words in a box
/// ```
///
/// A doc test may use `?` if it ends with an `Ok` value written like this:
///
/// ```
/// let n: i32 = "42".parse()?;
/// assert_eq!(n, 42);
/// # Ok::<(), std::num::ParseIntError>(())
/// ```
pub fn attributes_showcase() {}
