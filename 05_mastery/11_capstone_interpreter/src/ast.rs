//! Stage 2's output, the abstract syntax tree: the program's structure,
//! with the punctuation thrown away.
//!
//! An expression contains other expressions, so the enum is recursive. A
//! recursive enum needs a `Box` (or `Vec`, `Rc`...) at each level: without
//! one, `Expr` would have to contain itself directly and its size would be
//! infinite (exercise 1, E0072).

use std::fmt;
use std::rc::Rc;

use crate::error::Span;

/// An expression and where it starts (for runtime errors).
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Number(f64),
    Str(Rc<str>),
    Bool(bool),
    Nil,
    /// A name: `x`.
    Variable(String),
    /// `-x`, `!done`.
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// `a + b`, `a < b`... (but not `and`/`or`, which may skip `right`).
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `a and b`, `a or b`: `right` is only evaluated if needed.
    Logical {
        op: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `x = value`
    Assign {
        name: String,
        value: Box<Expr>,
    },
    /// `callee(arg, arg)`
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// `fn (a, b) { ... }`: a function value (a closure).
    Function(Rc<FunctionDecl>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

/// A function's parameters and body. Shared (`Rc`) between the AST and
/// every closure made from it, so calling a function never copies its body.
#[derive(Debug, PartialEq)]
pub struct FunctionDecl {
    /// `None` for an anonymous `fn (x) { ... }`.
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// A statement: something that is *done*, rather than a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let name = init;` (`init` defaults to nil)
    Let {
        name: String,
        init: Option<Expr>,
        span: Span,
    },
    /// An expression used for its effect: `print(x);`
    Expr(Expr),
    /// `{ ... }`: a new scope.
    Block(Vec<Stmt>),
    /// `if cond { ... } else ...` where the else branch is a block or another if.
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    /// `while cond { ... }`
    While { cond: Expr, body: Vec<Stmt> },
    /// `fn name(params) { ... }`
    Function(Rc<FunctionDecl>),
    /// `return value;`
    Return { value: Option<Expr>, span: Span },
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Expr { kind, span }
    }
}

impl BinaryOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
        }
    }
}

/// Prints an expression fully parenthesised, operator first (an
/// "S-expression"), so you can SEE how the parser grouped it:
/// `1 + 2 * 3` prints as `(+ 1 (* 2 3))`.
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Number(n) => write!(f, "{n}"),
            ExprKind::Str(s) => write!(f, "{s:?}"),
            ExprKind::Bool(b) => write!(f, "{b}"),
            ExprKind::Nil => write!(f, "nil"),
            ExprKind::Variable(name) => write!(f, "{name}"),
            ExprKind::Unary { op, operand } => {
                let symbol = match op {
                    UnaryOp::Negate => "-",
                    UnaryOp::Not => "!",
                };
                write!(f, "({symbol} {operand})")
            }
            ExprKind::Binary { op, left, right } => {
                write!(f, "({} {left} {right})", op.symbol())
            }
            ExprKind::Logical { op, left, right } => {
                let word = match op {
                    LogicalOp::And => "and",
                    LogicalOp::Or => "or",
                };
                write!(f, "({word} {left} {right})")
            }
            ExprKind::Assign { name, value } => write!(f, "(= {name} {value})"),
            ExprKind::Call { callee, args } => {
                write!(f, "(call {callee}")?;
                for arg in args {
                    write!(f, " {arg}")?;
                }
                write!(f, ")")
            }
            ExprKind::Function(decl) => write!(f, "(fn ({}) ...)", decl.params.join(" ")),
        }
    }
}
