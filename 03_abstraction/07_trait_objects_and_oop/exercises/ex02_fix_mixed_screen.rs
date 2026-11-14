//! Exercise 2: make it compile (E0308).
//!
//! `Screen<T>` is generic, so every component must be the SAME type `T`. This
//! screen wants a Label AND a Button. Change `Screen` to hold trait objects
//! (`Vec<Box<dyn Draw>>`, no type parameter), then update its `impl` and box
//! each component in `main`. Don't touch `Draw` or the components.
//!
//! Run: cargo run -p trait_objects_and_oop --features exercises --bin ex02_fix_mixed_screen

trait Draw {
    fn draw(&self) -> String;
}

struct Label {
    text: String,
}

struct Button {
    label: String,
}

impl Draw for Label {
    fn draw(&self) -> String {
        self.text.clone()
    }
}

impl Draw for Button {
    fn draw(&self) -> String {
        format!("[ {} ]", self.label)
    }
}

struct Screen<T: Draw> {
    components: Vec<T>,
}

impl<T: Draw> Screen<T> {
    fn render(&self) -> Vec<String> {
        self.components.iter().map(|c| c.draw()).collect()
    }
}

fn main() {
    let screen = Screen {
        components: vec![
            Label {
                text: "Save changes?".into(),
            },
            Button { label: "OK".into() },
        ],
    };
    for line in screen.render() {
        println!("{line}");
    }
}
