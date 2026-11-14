//! Book 18.2: trait objects let one collection hold values of DIFFERENT types
//! that share a behaviour. The Book's `gui` library, with text "drawing".
//! Run: cargo run -p trait_objects_and_oop --example gui_trait_objects

// ---------- The library ("gui" crate) ----------
// The library author does not know which components users will invent.
// They only promise: "anything that can draw itself may go on the screen".
pub trait Draw {
    fn draw(&self);
}

pub struct Screen {
    // `Box<dyn Draw>` is a TRAIT OBJECT: a pointer to some value of some type
    // that implements Draw, plus a vtable to find that type's `draw`.
    pub components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw(); // dynamic dispatch: looked up at run time
        }
    }
}

pub struct Button {
    pub width: u32,
    pub height: u32,
    pub label: String,
}

impl Draw for Button {
    fn draw(&self) {
        println!("[ {} ] ({}x{} button)", self.label, self.width, self.height);
    }
}

// ---------- The user's crate ----------
// A type the library has never heard of. Implementing Draw is enough to put
// it on the screen: no base class, no registration.
struct SelectBox {
    width: u32,
    height: u32,
    options: Vec<String>,
}

impl Draw for SelectBox {
    fn draw(&self) {
        println!(
            "<{}> ({}x{} select box)",
            self.options.join(" | "),
            self.width,
            self.height
        );
    }
}

// ---------- The generic alternative, and its limit ----------
// A generic Screen<T> is monomorphized (fast, static dispatch), but every
// component must be the SAME type T.
pub struct GenericScreen<T: Draw> {
    pub components: Vec<T>,
}

impl<T: Draw> GenericScreen<T> {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}

fn main() {
    let screen = Screen {
        components: vec![
            Box::new(SelectBox {
                width: 75,
                height: 10,
                options: vec!["Yes".into(), "Maybe".into(), "No".into()],
            }),
            Box::new(Button {
                width: 50,
                height: 10,
                label: "OK".into(),
            }),
        ],
    };
    println!("dyn screen (mixed types):");
    screen.run();

    // Only types that implement Draw are accepted:
    // let bad = Screen { components: vec![Box::new(String::from("Hi"))] };
    //   // E0277: the trait bound `String: Draw` is not satisfied

    let buttons = GenericScreen {
        components: vec![
            Button {
                width: 20,
                height: 5,
                label: "Save".into(),
            },
            Button {
                width: 20,
                height: 5,
                label: "Cancel".into(),
            },
        ],
    };
    println!("generic screen (one type only):");
    buttons.run();
    // let mixed = GenericScreen { components: vec![a_button, a_select_box] };
    //   // E0308: mismatched types, expected `Button`, found `SelectBox`

    // The price of `dyn`: a pointer to the data plus a pointer to the vtable.
    println!(
        "&Button is {} bytes, &dyn Draw is {} bytes",
        size_of::<&Button>(),
        size_of::<&dyn Draw>()
    );
}
