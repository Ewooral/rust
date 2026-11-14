//! Stage 4: errors from every stage, with line, column and a caret, the
//! way rustc reports yours.
//! Run: cargo run -p capstone_interpreter --example stage6_errors

use capstone_interpreter::Interpreter;

fn main() {
    let programs = [
        ("lexer", "let price = 5;\nlet total = price @ 2;"),
        ("parser", "fn area(w, h) {\n    return w * h\n}"),
        ("parser", "let x = 1;\nreturn x;"),
        (
            "runtime",
            "let items = 0;\nprint(\"average:\", 10 / items);",
        ),
        (
            "runtime",
            "let name = \"Ama\";\nprint(\"Hello \" + name + 1);",
        ),
        (
            "runtime",
            "fn greet(name) { print(\"hi\", name); }\ngreet();",
        ),
        ("runtime", "let n = 3;\nif n { print(\"yes\"); }"),
    ];
    for (stage, source) in programs {
        println!("--- found by the {stage}:");
        let mut kite = Interpreter::new(Vec::new());
        match kite.run(source) {
            Ok(()) => println!("(no error?)"),
            Err(e) => println!("{}\n", e.render(source)),
        }
    }
}
