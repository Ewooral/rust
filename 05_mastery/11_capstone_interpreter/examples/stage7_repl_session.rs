//! Stage 5: the REPL as a library type. Lines go in, events come out; the
//! binary only adds stdin, stdout and the prompts. Here a scripted session
//! shows what a person would see.
//! Run: cargo run -p capstone_interpreter --example stage7_repl_session
//! The real thing: cargo run -p capstone_interpreter

use capstone_interpreter::{Event, Repl};

fn main() {
    let session = [
        "let width = 6;",
        "width * 7",
        "fn area(w, h) {",
        "    return w * h;",
        "}",
        "area(width, 2)",
        "print(\"area is\", area(3, 4))",
        "width / 0",
        "(1 + ",
        "2)",
        "\"strings echo with quotes\"",
    ];
    let mut repl = Repl::new(std::io::stdout());
    for line in session {
        let prompt = if repl.is_continuing() { "... " } else { "> " };
        println!("{prompt}{line}");
        match repl.feed(line) {
            Event::NeedMore => {}
            Event::Done(None) => {}
            Event::Done(Some(value)) => println!("{value:?}"),
            Event::Error(text) => println!("{text}"),
        }
    }
}
