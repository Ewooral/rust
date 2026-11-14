//! Beyond the Book: supertraits, "to have this badge you need that one too",
//! and trait upcasting (`&dyn Sub` -> `&dyn Super`).
//! Run: cargo run -p trait_objects_and_oop --example supertraits
//!
//! `trait Widget: fmt::Display` means: every Widget must also be Display.
//! It is NOT inheritance: no fields or code are inherited. It's a promise
//! that lets Widget's own methods (and its users) rely on Display.

use std::fmt;

trait Named {
    fn name(&self) -> String;
}

// Widget requires BOTH Named and Display.
trait Widget: Named + fmt::Display {
    fn width(&self) -> usize;

    // A default method may use the supertraits' methods.
    fn framed(&self) -> String {
        let text = self.to_string(); // from Display
        format!(
            "+{}+\n|{text:^w$}|\n+{}+",
            "-".repeat(self.width()),
            "-".repeat(self.width()),
            w = self.width()
        )
    }
}

struct Button {
    label: String,
}

impl Named for Button {
    fn name(&self) -> String {
        "button".into()
    }
}

impl fmt::Display for Button {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.label)
    }
}

impl Widget for Button {
    fn width(&self) -> usize {
        self.label.len() + 6
    }
}
// Leave out `impl fmt::Display for Button` and `impl Widget for Button` fails:
//   // E0277: `Button` doesn't implement `std::fmt::Display`

struct Slider {
    value: u8,
}

impl Named for Slider {
    fn name(&self) -> String {
        "slider".into()
    }
}

impl fmt::Display for Slider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filled = usize::from(self.value / 10);
        write!(f, "{}{}", "#".repeat(filled), ".".repeat(10 - filled))
    }
}

impl Widget for Slider {
    fn width(&self) -> usize {
        14
    }
}

/// Through a `&dyn Widget` you can call Widget's methods AND the supertraits'
/// methods: they are all in the same vtable.
fn show(widget: &dyn Widget) {
    println!("{} ({} wide): {widget}", widget.name(), widget.width());
    println!("{}", widget.framed());
}

/// Trait upcasting: a `&dyn Widget` can be passed where a `&dyn Display` is
/// wanted, because Display is a supertrait of Widget.
fn print_all(items: &[&dyn fmt::Display]) {
    let joined: Vec<String> = items.iter().map(|d| d.to_string()).collect();
    println!("as plain Display: {}", joined.join("  "));
}

fn main() {
    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Button { label: "OK".into() }),
        Box::new(Slider { value: 70 }),
    ];
    for w in &widgets {
        show(w.as_ref());
    }

    // Upcast each `&dyn Widget` to `&dyn Display`.
    let displays: Vec<&dyn fmt::Display> = widgets
        .iter()
        .map(|w| w.as_ref() as &dyn fmt::Display)
        .collect();
    print_all(&displays);
}
