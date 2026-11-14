//! Exercise 6: supertraits and downcasting (Beyond the Book).
//!
//! `Widget` has two supertraits: `fmt::Display` (every widget can be printed)
//! and `Any` (every widget can be turned back into its real type).
//!
//! 1. `describe_all`: each widget's Display text plus its width, like
//!    "[OK] (4 wide)". Display is a supertrait, so `{}` works on a `dyn Widget`.
//! 2. `count_of::<T>`: how many widgets are really a `T`.
//! 3. `set_sliders`: set the value of every `Slider`, return how many changed.
//!
//! Hint: `Any` is a supertrait, so a `&dyn Widget` can be UPCAST to a
//! `&dyn Any` (`let any: &dyn Any = widget.as_ref();`), and `&mut` works the
//! same way. Then use `is::<T>()` or `downcast_mut::<T>()`.
//!
//! Test: cargo test -p trait_objects_and_oop --features exercises --bin ex06_widgets_supertrait_any

#![allow(unused_variables, unused_imports, dead_code)]

use std::any::Any;
use std::fmt;

trait Widget: fmt::Display + Any {
    fn width(&self) -> usize;
}

struct Button {
    label: String,
}

struct Slider {
    value: u8,
}

impl fmt::Display for Button {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.label)
    }
}

impl fmt::Display for Slider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}%>", self.value)
    }
}

impl Widget for Button {
    fn width(&self) -> usize {
        self.label.chars().count() + 2
    }
}

impl Widget for Slider {
    fn width(&self) -> usize {
        6
    }
}

fn describe_all(widgets: &[Box<dyn Widget>]) -> Vec<String> {
    todo!()
}

fn count_of<T: Widget>(widgets: &[Box<dyn Widget>]) -> usize {
    todo!()
}

fn set_sliders(widgets: &mut [Box<dyn Widget>], value: u8) -> usize {
    todo!()
}

fn sample() -> Vec<Box<dyn Widget>> {
    vec![
        Box::new(Button { label: "OK".into() }),
        Box::new(Slider { value: 50 }),
        Box::new(Button {
            label: "Cancel".into(),
        }),
        Box::new(Slider { value: 10 }),
    ]
}

fn main() {
    let mut widgets = sample();
    println!("{:?}", describe_all(&widgets));
    println!(
        "{} buttons, {} sliders",
        count_of::<Button>(&widgets),
        count_of::<Slider>(&widgets)
    );
    let changed = set_sliders(&mut widgets, 80);
    println!("changed {changed}: {:?}", describe_all(&widgets));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_uses_the_display_supertrait() {
        assert_eq!(
            describe_all(&sample()),
            [
                "[OK] (4 wide)",
                "<50%> (6 wide)",
                "[Cancel] (8 wide)",
                "<10%> (6 wide)"
            ]
        );
    }

    #[test]
    fn counts_each_concrete_type() {
        let widgets = sample();
        assert_eq!(count_of::<Button>(&widgets), 2);
        assert_eq!(count_of::<Slider>(&widgets), 2);
        assert_eq!(count_of::<Button>(&[]), 0);
    }

    #[test]
    fn sliders_can_be_found_and_changed() {
        let mut widgets = sample();
        assert_eq!(set_sliders(&mut widgets, 80), 2);
        assert_eq!(describe_all(&widgets)[1], "<80%> (6 wide)");
        assert_eq!(describe_all(&widgets)[3], "<80%> (6 wide)");
        assert_eq!(describe_all(&widgets)[0], "[OK] (4 wide)");
    }
}
