//! # Kite: a small language, and a tree-walking interpreter for it
//!
//! The capstone interpreter. A Kite program goes through three stages:
//!
//! ```text
//! source text --lexer--> tokens --parser--> AST --interpreter--> output
//!   "1 + 2"       [1] [+] [2]      (+ 1 2)          3
//! ```
//!
//! ```
//! use capstone_interpreter::Interpreter;
//!
//! let program = r#"
//!     fn fib(n) {
//!         if n < 2 { return n; }
//!         return fib(n - 1) + fib(n - 2);
//!     }
//!     print("fib(10) =", fib(10));
//! "#;
//! let mut kite = Interpreter::new(Vec::new());
//! kite.run(program)?;
//! assert_eq!(String::from_utf8(kite.into_output()).unwrap(), "fib(10) = 55\n");
//! # Ok::<(), capstone_interpreter::Error>(())
//! ```

pub mod ast;
pub mod env;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod token;
pub mod value;

pub use ast::{Expr, ExprKind, Stmt};
pub use env::Env;
pub use error::{Error, ErrorKind, Span};
pub use interpreter::{DEFAULT_MAX_CALL_DEPTH, Interpreter};
pub use lexer::lex;
pub use parser::{Parser, parse, parse_expr};
pub use repl::{Event, Repl};
pub use token::{Token, TokenKind};
pub use value::{Builtin, BuiltinFn, Value};
