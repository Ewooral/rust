//! Solution to exercise 2: make it compile (E0382) by sharing ownership.
//!
//! `Rc::clone` doesn't copy the teacher: it adds one to a counter and hands
//! back another pointer to the same `Teacher`. The count is 3: the
//! `teacher` variable and the two classes. The teacher is freed only when
//! all three are gone.
//!
//! Run: cargo run -p smart_pointers --features solutions --bin sol02_fix_shared_owner

use std::rc::Rc;

#[derive(Debug)]
struct Teacher {
    name: String,
}

#[derive(Debug)]
struct Class {
    subject: &'static str,
    teacher: Rc<Teacher>,
}

fn main() {
    let teacher = Rc::new(Teacher {
        name: String::from("Ms Mensah"),
    });
    let maths = Class {
        subject: "maths",
        teacher: Rc::clone(&teacher),
    };
    let art = Class {
        subject: "art",
        teacher: Rc::clone(&teacher),
    };
    for class in [&maths, &art] {
        println!("{} is taught by {}", class.subject, class.teacher.name);
    }
    println!("owners of the teacher: {}", Rc::strong_count(&teacher));
    drop(maths);
    println!("after the maths class ends: {}", Rc::strong_count(&teacher));
}
