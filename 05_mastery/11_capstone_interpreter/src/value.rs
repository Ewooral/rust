//! Runtime values: what Kite expressions evaluate to.

use std::cell::RefCell;
use std::fmt;
use std::io::Write;
use std::rc::Rc;

use crate::ast::FunctionDecl;
use crate::env::Env;

/// A Kite value. Cloning is cheap: strings and functions are behind `Rc`,
/// so a clone copies a pointer, not the text or the function.
#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Str(Rc<str>),
    /// A function written in Kite, with the scope it was created in.
    Function(Rc<Closure>),
    /// A function written in Rust, such as `print`.
    Builtin(Builtin),
}

/// A function plus the environment it was defined in. Keeping that
/// environment alive is what lets an inner function use the outer
/// function's variables after the outer function has returned.
pub struct Closure {
    pub decl: Rc<FunctionDecl>,
    pub env: Rc<RefCell<Env>>,
}

/// The signature of a built-in function: the arguments, and somewhere to
/// write output. An `Err` message becomes a runtime error at the call.
pub type BuiltinFn = fn(&[Value], &mut dyn Write) -> Result<Value, String>;

/// A function implemented in Rust.
#[derive(Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFn,
}

impl Value {
    /// The type's name, for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::Str(_) => "string",
            Value::Function(_) | Value::Builtin(_) => "function",
        }
    }
}

impl PartialEq for Value {
    /// Values of different types are never equal. Functions are equal only
    /// to themselves (the same `Rc`).
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            (Value::Builtin(a), Value::Builtin(b)) => a.name == b.name,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    /// How `print` shows a value. Whole numbers print without `.0`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Function(closure) => match &closure.decl.name {
                Some(name) => write!(f, "<fn {name}>"),
                None => write!(f, "<fn>"),
            },
            Value::Builtin(b) => write!(f, "<builtin {}>", b.name),
        }
    }
}

impl fmt::Debug for Value {
    /// Written by hand: a derived `Debug` would print a closure's
    /// environment, which can contain the closure itself, forever.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{s:?}"),
            other => write!(f, "{other}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_like_people_write_them() {
        assert_eq!(Value::Number(3.0).to_string(), "3");
        assert_eq!(Value::Number(0.5).to_string(), "0.5");
        assert_eq!(Value::Number(-2.0).to_string(), "-2");
    }

    #[test]
    fn equality_never_mixes_types() {
        assert_eq!(Value::Number(1.0), Value::Number(1.0));
        assert_ne!(Value::Number(1.0), Value::Str("1".into()));
        assert_ne!(Value::Bool(false), Value::Nil);
        assert_eq!(Value::Str("a".into()), Value::Str("a".into()));
    }

    #[test]
    fn debug_quotes_strings() {
        assert_eq!(format!("{:?}", Value::Str("hi".into())), "\"hi\"");
        assert_eq!(format!("{:?}", Value::Bool(true)), "true");
    }
}
