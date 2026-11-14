//! Exercise 2: make it compile (E0382) by sharing ownership with `Rc`.
//!
//! Two classes share ONE teacher. A `Box` has exactly one owner: once the
//! maths class owns the box, the art class can't have it too.
//!
//! Change `Box<Teacher>` to `Rc<Teacher>` (and `Box::new` to `Rc::new`), and
//! give each class its own `Rc::clone(&teacher)`. Then print
//! `Rc::strong_count(&teacher)`. What number do you get, and why?
//!
//! Run: cargo run -p smart_pointers --features exercises --bin ex02_fix_shared_owner

#[derive(Debug)]
struct Teacher {
    name: String,
}

#[derive(Debug)]
struct Class {
    subject: &'static str,
    teacher: Box<Teacher>,
}

fn main() {
    let teacher = Box::new(Teacher {
        name: String::from("Ms Mensah"),
    });
    let maths = Class {
        subject: "maths",
        teacher,
    };
    let art = Class {
        subject: "art",
        teacher,
    };
    for class in [&maths, &art] {
        println!("{} is taught by {}", class.subject, class.teacher.name);
    }
}
