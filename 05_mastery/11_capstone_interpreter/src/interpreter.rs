//! Stage 3, the evaluator: walks the AST and does what it says.
//!
//! "Tree-walking" means exactly that: `eval` looks at a node, evaluates its
//! children (recursively), and combines the results. It's the simplest kind
//! of interpreter; faster ones compile the tree to bytecode first.

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expr, ExprKind, LogicalOp, Stmt, UnaryOp};
use crate::env::Env;
use crate::error::{Error, Span};
use crate::lexer::lex;
use crate::parser::{Parser, parse};
use crate::value::{Builtin, BuiltinFn, Closure, Value};

/// How deep Kite calls may nest, by default, before we stop with an error
/// instead of overflowing Rust's own stack.
///
/// Each Kite call is several nested Rust calls (`call` -> `exec_block` ->
/// `exec` -> `eval` -> ...), and in a debug build those frames are big:
/// measured here, 100 nested Kite calls already overflowed a 1 MiB stack
/// (the size of Windows' main thread). So the binary runs the interpreter
/// on a thread with a 64 MiB stack, where 1000 calls fit easily; code on a
/// smaller stack should lower the limit with [`Interpreter::set_max_depth`].
pub const DEFAULT_MAX_CALL_DEPTH: usize = 1000;

/// What running a statement did: carry on, or `return` from the function.
enum Flow {
    Normal,
    Return(Value),
}

/// Runs Kite programs. `print` writes to `out`, so tests can pass a
/// `Vec<u8>` and the binary passes stdout.
pub struct Interpreter<W: Write> {
    globals: Rc<RefCell<Env>>,
    out: W,
    depth: usize,
    max_depth: usize,
}

impl<W: Write> Interpreter<W> {
    /// An interpreter with the built-in functions `print` and `str`.
    pub fn new(out: W) -> Self {
        let mut interpreter = Interpreter {
            globals: Env::new_global(),
            out,
            depth: 0,
            max_depth: DEFAULT_MAX_CALL_DEPTH,
        };
        interpreter.define_builtin("print", builtin_print);
        interpreter.define_builtin("str", builtin_str);
        interpreter
    }

    /// Changes how deeply calls may nest (see [`DEFAULT_MAX_CALL_DEPTH`]).
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }

    /// Adds a function written in Rust (exercise 5 adds `len`).
    pub fn define_builtin(&mut self, name: &'static str, func: BuiltinFn) {
        self.globals
            .borrow_mut()
            .define(name, Value::Builtin(Builtin { name, func }));
    }

    /// Lexes, parses and runs a program. Variables and functions stay
    /// defined for the next call (that's how the REPL remembers things).
    ///
    /// ```
    /// use capstone_interpreter::Interpreter;
    /// let mut kite = Interpreter::new(Vec::new());
    /// kite.run("fn twice(x) { return x * 2; } print(twice(21));")?;
    /// assert_eq!(String::from_utf8(kite.into_output()).unwrap(), "42\n");
    /// # Ok::<(), capstone_interpreter::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The first syntax error, or the runtime error that stopped the program.
    pub fn run(&mut self, source: &str) -> Result<(), Error> {
        let program = parse(source)?;
        self.execute(&program).map(|_| ())
    }

    /// Like [`Interpreter::run`], but the final expression statement may
    /// leave out its `;`, and its value is returned (for the REPL to show).
    ///
    /// # Errors
    ///
    /// As `run`. A syntax error with `at_end` set means the input stopped
    /// in the middle of something, such as an unclosed `{`.
    pub fn run_repl(&mut self, source: &str) -> Result<Option<Value>, Error> {
        let program = Parser::new(lex(source)?).repl_mode().program()?;
        self.execute(&program)
    }

    /// Runs top-level statements; returns the value of the last one if it
    /// was an expression statement.
    fn execute(&mut self, program: &[Stmt]) -> Result<Option<Value>, Error> {
        let globals = Rc::clone(&self.globals);
        let mut last = None;
        for statement in program {
            last = None;
            if let Stmt::Expr(expr) = statement {
                last = Some(self.eval(expr, &globals)?);
            } else {
                // `return` at the top level is a syntax error, so Flow is Normal.
                self.exec(statement, &globals)?;
            }
        }
        self.out
            .flush()
            .map_err(|e| Error::runtime(format!("cannot write output: {e}"), Span::default()))?;
        Ok(last)
    }

    /// The output written so far.
    pub fn output(&self) -> &W {
        &self.out
    }

    /// Gives the output writer back.
    pub fn into_output(self) -> W {
        self.out
    }

    // ---------- statements ----------

    fn exec(&mut self, stmt: &Stmt, env: &Rc<RefCell<Env>>) -> Result<Flow, Error> {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval(expr, env)?;
            }
            Stmt::Let { name, init, .. } => {
                let value = match init {
                    Some(expr) => self.eval(expr, env)?,
                    None => Value::Nil,
                };
                // Evaluate first, THEN borrow mutably: holding a borrow
                // while evaluating would panic if the expression reads env.
                env.borrow_mut().define(name.clone(), value);
            }
            Stmt::Block(body) => return self.exec_block(body, Env::child(env)),
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                if self.condition(cond, env, "if")? {
                    return self.exec_block(then_branch, Env::child(env));
                } else if let Some(other) = else_branch {
                    return self.exec(other, env);
                }
            }
            Stmt::While { cond, body } => {
                while self.condition(cond, env, "while")? {
                    if let Flow::Return(value) = self.exec_block(body, Env::child(env))? {
                        return Ok(Flow::Return(value));
                    }
                }
            }
            Stmt::Function(decl) => {
                let closure = Closure {
                    decl: Rc::clone(decl),
                    env: Rc::clone(env),
                };
                let name = decl.name.clone().unwrap_or_default();
                env.borrow_mut()
                    .define(name, Value::Function(Rc::new(closure)));
            }
            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(expr) => self.eval(expr, env)?,
                    None => Value::Nil,
                };
                return Ok(Flow::Return(value));
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_block(&mut self, body: &[Stmt], env: Rc<RefCell<Env>>) -> Result<Flow, Error> {
        for stmt in body {
            if let Flow::Return(value) = self.exec(stmt, &env)? {
                return Ok(Flow::Return(value)); // stop the block, pass it up
            }
        }
        Ok(Flow::Normal)
    }

    /// Evaluates an `if`/`while` condition, which must be a bool.
    fn condition(
        &mut self,
        cond: &Expr,
        env: &Rc<RefCell<Env>>,
        what: &str,
    ) -> Result<bool, Error> {
        match self.eval(cond, env)? {
            Value::Bool(b) => Ok(b),
            other => Err(Error::runtime(
                format!(
                    "the condition of '{what}' must be a bool, not {}",
                    other.type_name()
                ),
                cond.span,
            )),
        }
    }

    // ---------- expressions ----------

    /// Evaluates an expression to a value.
    pub fn eval(&mut self, expr: &Expr, env: &Rc<RefCell<Env>>) -> Result<Value, Error> {
        match &expr.kind {
            ExprKind::Number(n) => Ok(Value::Number(*n)),
            ExprKind::Str(s) => Ok(Value::Str(Rc::clone(s))),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Nil => Ok(Value::Nil),
            ExprKind::Variable(name) => env
                .borrow()
                .get(name)
                .ok_or_else(|| Error::runtime(format!("undefined variable '{name}'"), expr.span)),
            ExprKind::Assign { name, value } => {
                let value = self.eval(value, env)?;
                if env.borrow_mut().assign(name, value.clone()) {
                    Ok(value)
                } else {
                    Err(Error::runtime(
                        format!("undefined variable '{name}' (declare it with 'let' first)"),
                        expr.span,
                    ))
                }
            }
            ExprKind::Unary { op, operand } => {
                let value = self.eval(operand, env)?;
                match (op, value) {
                    (UnaryOp::Negate, Value::Number(n)) => Ok(Value::Number(-n)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    (UnaryOp::Negate, other) => Err(Error::runtime(
                        format!("cannot negate a {}", other.type_name()),
                        expr.span,
                    )),
                    (UnaryOp::Not, other) => Err(Error::runtime(
                        format!("'!' needs a bool, not a {}", other.type_name()),
                        expr.span,
                    )),
                }
            }
            ExprKind::Binary { op, left, right } => {
                let left = self.eval(left, env)?;
                let right = self.eval(right, env)?;
                binary(*op, left, right).map_err(|message| Error::runtime(message, expr.span))
            }
            ExprKind::Logical { op, left, right } => {
                let left_value = self.logic_operand(left, env, *op)?;
                // Short-circuit: `false and ...` and `true or ...` are
                // decided by the left side alone; the right is never run.
                match (op, left_value) {
                    (LogicalOp::And, false) => Ok(Value::Bool(false)),
                    (LogicalOp::Or, true) => Ok(Value::Bool(true)),
                    _ => Ok(Value::Bool(self.logic_operand(right, env, *op)?)),
                }
            }
            ExprKind::Call { callee, args } => {
                let callee = self.eval(callee, env)?;
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.eval(arg, env)?);
                }
                self.call(callee, values, expr.span)
            }
            ExprKind::Function(decl) => Ok(Value::Function(Rc::new(Closure {
                decl: Rc::clone(decl),
                env: Rc::clone(env),
            }))),
        }
    }

    fn logic_operand(
        &mut self,
        expr: &Expr,
        env: &Rc<RefCell<Env>>,
        op: LogicalOp,
    ) -> Result<bool, Error> {
        match self.eval(expr, env)? {
            Value::Bool(b) => Ok(b),
            other => {
                let word = if op == LogicalOp::And { "and" } else { "or" };
                Err(Error::runtime(
                    format!("'{word}' needs bools, not a {}", other.type_name()),
                    expr.span,
                ))
            }
        }
    }

    /// Calls a function value with already-evaluated arguments.
    fn call(&mut self, callee: Value, args: Vec<Value>, span: Span) -> Result<Value, Error> {
        match callee {
            Value::Builtin(builtin) => (builtin.func)(&args, &mut self.out)
                .map_err(|message| Error::runtime(format!("{}: {message}", builtin.name), span)),
            Value::Function(closure) => {
                let decl = &closure.decl;
                if args.len() != decl.params.len() {
                    let name = decl.name.as_deref().unwrap_or("this function");
                    let plural = if decl.params.len() == 1 { "" } else { "s" };
                    return Err(Error::runtime(
                        format!(
                            "{name} takes {} argument{plural} but was given {}",
                            decl.params.len(),
                            args.len()
                        ),
                        span,
                    ));
                }
                if self.depth >= self.max_depth {
                    return Err(Error::runtime(
                        format!(
                            "too much recursion (more than {} nested calls)",
                            self.max_depth
                        ),
                        span,
                    ));
                }
                // The new scope's parent is where the function was DEFINED
                // (closure.env), not where it is called: that's a closure.
                let scope = Env::child(&closure.env);
                for (param, arg) in decl.params.iter().zip(args) {
                    scope.borrow_mut().define(param.clone(), arg);
                }
                self.depth += 1;
                let flow = self.exec_block(&decl.body, scope);
                self.depth -= 1;
                match flow? {
                    Flow::Return(value) => Ok(value),
                    Flow::Normal => Ok(Value::Nil),
                }
            }
            other => Err(Error::runtime(
                format!("can only call functions, not a {}", other.type_name()),
                span,
            )),
        }
    }
}

/// `+ - * / == != < <= > >=` on two values.
fn binary(op: BinaryOp, left: Value, right: Value) -> Result<Value, String> {
    use Value::{Bool, Number, Str};
    Ok(match (op, &left, &right) {
        (BinaryOp::Equal, _, _) => Bool(left == right),
        (BinaryOp::NotEqual, _, _) => Bool(left != right),
        (BinaryOp::Add, Number(a), Number(b)) => Number(a + b),
        (BinaryOp::Add, Str(a), Str(b)) => Str(format!("{a}{b}").into()),
        (BinaryOp::Subtract, Number(a), Number(b)) => Number(a - b),
        (BinaryOp::Multiply, Number(a), Number(b)) => Number(a * b),
        (BinaryOp::Divide, Number(_), Number(b)) if *b == 0.0 => {
            return Err("division by zero".into());
        }
        (BinaryOp::Divide, Number(a), Number(b)) => Number(a / b),
        (BinaryOp::Less, Number(a), Number(b)) => Bool(a < b),
        (BinaryOp::LessEqual, Number(a), Number(b)) => Bool(a <= b),
        (BinaryOp::Greater, Number(a), Number(b)) => Bool(a > b),
        (BinaryOp::GreaterEqual, Number(a), Number(b)) => Bool(a >= b),
        (BinaryOp::Less, Str(a), Str(b)) => Bool(a < b),
        (BinaryOp::LessEqual, Str(a), Str(b)) => Bool(a <= b),
        (BinaryOp::Greater, Str(a), Str(b)) => Bool(a > b),
        (BinaryOp::GreaterEqual, Str(a), Str(b)) => Bool(a >= b),
        (op, _, _) => {
            return Err(format!(
                "cannot use '{}' on a {} and a {}",
                op.symbol(),
                left.type_name(),
                right.type_name()
            ));
        }
    })
}

/// `print(a, b, ...)`: the values separated by spaces, then a new line.
fn builtin_print(args: &[Value], out: &mut dyn Write) -> Result<Value, String> {
    let text: Vec<String> = args.iter().map(Value::to_string).collect();
    writeln!(out, "{}", text.join(" ")).map_err(|e| e.to_string())?;
    Ok(Value::Nil)
}

/// `str(x)`: any value as a string, for joining with `+`.
fn builtin_str(args: &[Value], _out: &mut dyn Write) -> Result<Value, String> {
    match args {
        [value] => Ok(Value::Str(value.to_string().into())),
        _ => Err(format!("takes 1 argument but was given {}", args.len())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs a program and returns what it printed.
    fn output(source: &str) -> String {
        let mut kite = Interpreter::new(Vec::new());
        kite.run(source)
            .unwrap_or_else(|e| panic!("{}", e.render(source)));
        String::from_utf8(kite.into_output()).unwrap()
    }

    fn error(source: &str) -> String {
        let mut kite = Interpreter::new(Vec::new());
        kite.run(source).unwrap_err().to_string()
    }

    #[test]
    fn arithmetic_with_precedence() {
        assert_eq!(
            output("print(1 + 2 * 3, (1 + 2) * 3, 10 - 3 - 2, -2 * 3, 7 / 2);"),
            "7 9 5 -6 3.5\n"
        );
    }

    #[test]
    fn strings_bools_and_nil() {
        assert_eq!(
            output(r#"print("a" + "b", "n = " + str(4), !true, nil, "b" > "a");"#),
            "ab n = 4 false nil true\n"
        );
    }

    #[test]
    fn comparisons_and_equality() {
        assert_eq!(
            output("print(1 < 2, 2 <= 2, 3 > 4, 1 == 1, 1 == \"1\", nil != false);"),
            "true true false true false true\n"
        );
    }

    #[test]
    fn and_or_short_circuit() {
        // If `or` evaluated its right side, undefined_name would be an error.
        assert_eq!(
            output("print(true or undefined_name, false and undefined_name);"),
            "true false\n"
        );
    }

    #[test]
    fn variables_scopes_and_shadowing() {
        let program = "
            let x = 1;
            {
                let x = 2;      // a new x, only inside the block
                print(x);
            }
            print(x);
            x = x + 10;         // assigns the outer x
            print(x);
        ";
        assert_eq!(output(program), "2\n1\n11\n");
    }

    #[test]
    fn if_else_chains_and_while() {
        let program = "
            let i = 0;
            while i < 5 {
                if i == 0 { print(\"zero\"); }
                else if i < 3 { print(\"small\", i); }
                else { print(\"big\", i); }
                i = i + 1;
            }
        ";
        assert_eq!(output(program), "zero\nsmall 1\nsmall 2\nbig 3\nbig 4\n");
    }

    #[test]
    fn functions_and_recursion() {
        let program = "
            fn fib(n) {
                if n < 2 { return n; }
                return fib(n - 1) + fib(n - 2);
            }
            print(fib(20));
            fn nothing() { }
            print(nothing());
        ";
        assert_eq!(output(program), "6765\nnil\n");
    }

    #[test]
    fn closures_remember_their_scope() {
        let program = "
            fn make_counter() {
                let count = 0;
                return fn () {
                    count = count + 1;
                    return count;
                };
            }
            let a = make_counter();
            let b = make_counter();
            a(); a();
            print(a(), b());
        ";
        assert_eq!(output(program), "3 1\n");
    }

    #[test]
    fn functions_are_values() {
        let program = "
            fn apply(f, x) { return f(x); }
            let square = fn (n) { return n * n; };
            print(apply(square, 7), square, apply);
        ";
        assert_eq!(output(program), "49 <fn> <fn apply>\n");
    }

    #[test]
    fn return_leaves_loops_and_blocks() {
        let program = "
            fn first_over(limit) {
                let i = 0;
                while true {
                    { if i * i > limit { return i; } }
                    i = i + 1;
                }
            }
            print(first_over(50));
        ";
        assert_eq!(output(program), "8\n");
    }

    #[test]
    fn runtime_errors_have_positions() {
        assert_eq!(
            error("let x = 1;\nprint(y);"),
            "runtime error at line 2, column 7: undefined variable 'y'"
        );
        assert_eq!(
            error("print(1 + \"a\");"),
            "runtime error at line 1, column 9: cannot use '+' on a number and a string"
        );
        assert_eq!(
            error("let n = 0;\nprint(10 / n);"),
            "runtime error at line 2, column 10: division by zero"
        );
        assert_eq!(
            error("if 1 { }"),
            "runtime error at line 1, column 4: the condition of 'if' must be a bool, not number"
        );
        assert_eq!(
            error("fn f(a) { }\nf(1, 2);"),
            "runtime error at line 2, column 2: f takes 1 argument but was given 2"
        );
        assert_eq!(
            error("let x = 3;\nx();"),
            "runtime error at line 2, column 2: can only call functions, not a number"
        );
        assert_eq!(
            error("y = 1;"),
            "runtime error at line 1, column 3: undefined variable 'y' (declare it with 'let' first)"
        );
        assert_eq!(
            error("str(1, 2);"),
            "runtime error at line 1, column 4: str: takes 1 argument but was given 2"
        );
    }

    #[test]
    fn runaway_recursion_is_an_error_not_a_crash() {
        // Test threads have small stacks, so use a small limit here; the
        // binary's 64 MiB thread is tested with the default in tests/cli.rs.
        let mut kite = Interpreter::new(Vec::new());
        kite.set_max_depth(50);
        let err = kite
            .run("fn forever(n) { return forever(n + 1); }\nforever(0);")
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "runtime error at line 1, column 31: too much recursion (more than 50 nested calls)"
        );
    }

    #[test]
    fn the_interpreter_remembers_between_runs() {
        let mut kite = Interpreter::new(Vec::new());
        kite.run("let total = 1;").unwrap();
        kite.run("total = total + 1;").unwrap();
        assert_eq!(
            kite.run_repl("total * 10").unwrap(),
            Some(Value::Number(20.0))
        );
        assert_eq!(kite.run_repl("let z = 1;").unwrap(), None);
    }
}
