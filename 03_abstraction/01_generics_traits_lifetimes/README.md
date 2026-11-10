# Generics Traits Lifetimes

**The Rust Book, chapter 10:** [Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)

Run it: `cargo run -p generics_traits_lifetimes`  ·  Test it: `cargo test -p generics_traits_lifetimes`

## Concepts (from the Book)

- [ ] [10.1 Generic Data Types](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [ ] [10.2 Defining Shared Behavior with Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [ ] [10.3 Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)

## Beyond the Book

- [ ] Static dispatch (generics) vs dynamic dispatch (`dyn Trait`, vtables)
- [ ] `impl Trait` in argument and return position
- [ ] Lifetime bounds (`where 'a: 'b`), the anonymous lifetime `'_`, `'static`
- [ ] Conversion traits: `From`/`Into`, `TryFrom`/`TryInto`, `AsRef`/`Borrow`
- [ ] Common std traits: `Display`, `Default`, `PartialOrd`/`Ord`, `Hash`, operator traits
- [ ] Const generics (`struct Matrix<const N: usize>`)
- [ ] Generic associated types (GATs)
- [ ] Higher-ranked trait bounds (`for<'a> Fn(&'a T)`)
- [ ] Variance, subtyping and `PhantomData`
- [ ] Coherence and the orphan rule in practice
