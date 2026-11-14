//! Solution to exercise 5: extend the REAL Kite with a built-in function, `len`.
//!
//! Built-ins are Rust functions with this signature (src/value.rs):
//!
//!   fn(&[Value], &mut dyn Write) -> Result<Value, String>
//!
//! and `Interpreter::define_builtin` makes one callable from Kite. An `Err`
//! message becomes a runtime error at the call, with its line and column,
//! prefixed with the function's name ("len: ...").
//!
//! Implement `len`:
//! - `len("héllo")` is 5: count CHARACTERS (`chars().count()`), not bytes
//!   (`"héllo".len()` would say 6).
//! - Exactly one argument, or `Err("takes 1 argument but was given 2")`.
//! - A string, or `Err("needs a string, not a number")` (use `type_name()`).
//!
//! A slice pattern (`[Value::Str(text)]`) checks the count and the type in
//! one `match`. The Rust side never sees line numbers: the interpreter adds
//! the position of the call, so every built-in gets good errors for free.
//!
//! Test: cargo test -p capstone_interpreter --features solutions --bin sol05_len_builtin

use std::io::Write;

use capstone_interpreter::{Interpreter, Value};

/// `len(text)`: the number of characters in a string.
fn len(args: &[Value], _out: &mut dyn Write) -> Result<Value, String> {
    match args {
        [Value::Str(text)] => Ok(Value::Number(text.chars().count() as f64)),
        [other] => Err(format!("needs a string, not a {}", other.type_name())),
        _ => Err(format!("takes 1 argument but was given {}", args.len())),
    }
}

/// An interpreter with `len` added.
fn kite() -> Interpreter<Vec<u8>> {
    let mut interpreter = Interpreter::new(Vec::new());
    interpreter.define_builtin("len", len);
    interpreter
}

fn main() {
    let program = r#"
        let word = "héllo";
        print(word, "has", len(word), "characters");
        print("doubled:", len(word + word));
    "#;
    let mut interpreter = kite();
    match interpreter.run(program) {
        Ok(()) => print!("{}", String::from_utf8_lossy(&interpreter.into_output())),
        Err(e) => eprintln!("{}", e.render(program)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(program: &str) -> Result<String, String> {
        let mut interpreter = kite();
        interpreter.set_max_depth(50); // test threads have small stacks
        interpreter.run(program).map_err(|e| e.to_string())?;
        Ok(String::from_utf8(interpreter.into_output()).unwrap())
    }

    #[test]
    fn counts_characters_not_bytes() {
        assert_eq!(output("print(len(\"héllo\"));").unwrap(), "5\n");
        assert_eq!(output("print(len(\"\"));").unwrap(), "0\n");
        assert_eq!(
            output("let s = \"abc\";\nprint(len(s + s));").unwrap(),
            "6\n"
        );
    }

    #[test]
    fn works_inside_kite_functions_and_conditions() {
        let program = "
            fn longer(a, b) {
                if len(a) >= len(b) { return a; }
                return b;
            }
            print(longer(\"kite\", \"interpreter\"));
        ";
        assert_eq!(output(program).unwrap(), "interpreter\n");
    }

    #[test]
    fn wrong_arguments_are_runtime_errors_with_a_position() {
        assert_eq!(
            output("print(len(42));").unwrap_err(),
            "runtime error at line 1, column 10: len: needs a string, not a number"
        );
        assert_eq!(
            output("let n = len(\"a\", \"b\");").unwrap_err(),
            "runtime error at line 1, column 12: len: takes 1 argument but was given 2"
        );
        assert_eq!(
            output("len();").unwrap_err(),
            "runtime error at line 1, column 4: len: takes 1 argument but was given 0"
        );
    }
}
