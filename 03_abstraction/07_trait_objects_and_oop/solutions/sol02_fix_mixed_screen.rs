//! Solution to exercise 2: make it compile (E0308).
//!
//! `Screen<T>` is generic, so every component must be the SAME type `T`. This
//! screen wants a Label AND a Button. Change `Screen` to hold trait objects
//! (`Vec<Box<dyn Draw>>`, no type parameter), then update its `impl` and box
//! each component in `main`. Don't touch `Draw` or the components.
//!
//! Run: cargo run -p trait_objects_and_oop --features solutions --bin sol02_fix_mixed_screen

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

// No type parameter: each element may be a different type, as long as it
// implements Draw. The price is a Box per component and a vtable call.
struct Screen {
    components: Vec<Box<dyn Draw>>,
}

impl Screen {
    fn render(&self) -> Vec<String> {
        self.components.iter().map(|c| c.draw()).collect()
    }
}

fn main() {
    let screen = Screen {
        components: vec![
            Box::new(Label {
                text: "Save changes?".into(),
            }),
            Box::new(Button { label: "OK".into() }),
        ],
    };
    for line in screen.render() {
        println!("{line}");
    }
}
