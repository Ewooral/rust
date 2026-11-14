//! Stage 2, the parser: tokens -> AST.
//!
//! Statements are parsed by **recursive descent**: one method per kind of
//! statement, each calling the others. Expressions are parsed by **Pratt
//! parsing** ("precedence climbing"): every operator has a *binding power*,
//! and an operator only takes the expression on its right if it binds more
//! tightly than the operator that came before it.

use std::rc::Rc;

use crate::ast::{BinaryOp, Expr, ExprKind, FunctionDecl, LogicalOp, Stmt, UnaryOp};
use crate::error::Error;
use crate::lexer::lex;
use crate::token::{Token, TokenKind};

/// Lexes and parses a whole program.
///
/// ```
/// use capstone_interpreter::parse;
/// let program = parse("let x = 1 + 2 * 3;")?;
/// assert_eq!(program.len(), 1);
/// # Ok::<(), capstone_interpreter::Error>(())
/// ```
///
/// # Errors
///
/// The first syntax error, with its position.
pub fn parse(source: &str) -> Result<Vec<Stmt>, Error> {
    Parser::new(lex(source)?).program()
}

/// Parses one expression on its own (for tests and the examples).
///
/// ```
/// use capstone_interpreter::parse_expr;
/// assert_eq!(parse_expr("1 + 2 * 3")?.to_string(), "(+ 1 (* 2 3))");
/// assert_eq!(parse_expr("(1 + 2) * 3")?.to_string(), "(* (+ 1 2) 3)");
/// # Ok::<(), capstone_interpreter::Error>(())
/// ```
///
/// # Errors
///
/// A syntax error, including leftover tokens after the expression.
pub fn parse_expr(source: &str) -> Result<Expr, Error> {
    let mut parser = Parser::new(lex(source)?);
    let expr = parser.expression()?;
    parser.expect(&TokenKind::Eof, "the end of the expression")?;
    Ok(expr)
}

/// How tightly each infix operator binds, as (left, right) powers.
///
/// Higher binds tighter: `*` (13) beats `+` (11), so `1 + 2 * 3` is
/// `1 + (2 * 3)`. Right power = left + 1 makes an operator
/// left-associative (`10 - 3 - 2` is `(10 - 3) - 2`); right power below
/// left makes it right-associative (`a = b = 1` is `a = (b = 1)`).
fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    Some(match kind {
        TokenKind::Equal => (2, 1),
        TokenKind::Or => (3, 4),
        TokenKind::And => (5, 6),
        TokenKind::EqualEqual | TokenKind::BangEqual => (7, 8),
        TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => {
            (9, 10)
        }
        TokenKind::Plus | TokenKind::Minus => (11, 12),
        TokenKind::Star | TokenKind::Slash => (13, 14),
        _ => return None,
    })
}

/// `-x` and `!x` bind tighter than any infix operator: `-2 * 3` is `(-2) * 3`.
const PREFIX_POWER: u8 = 15;
/// A call `f(x)` binds tightest of all: `-f(x)` is `-(f(x))`.
const CALL_POWER: u8 = 17;

/// The parser's state: the tokens and how far it has read.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// How many functions we're inside: `return` is only allowed above 0.
    function_depth: usize,
    /// In the REPL, the last expression statement may leave out its `;`.
    repl: bool,
}

impl Parser {
    /// A parser for `tokens`, which must end with `Eof` (as `lex` ensures).
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            function_depth: 0,
            repl: false,
        }
    }

    /// Lets the final expression statement leave out its `;` (`1 + 2` in the REPL).
    pub fn repl_mode(mut self) -> Self {
        self.repl = true;
        self
    }

    // ---------- moving through the tokens ----------

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        &self.peek().kind == kind
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if token.kind != TokenKind::Eof {
            self.pos += 1;
        }
        token
    }

    fn matches(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Takes a token of `kind`, or fails with "expected {what}".
    fn expect(&mut self, kind: &TokenKind, what: &str) -> Result<Token, Error> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error_here(format!("expected {what}, found {}", self.peek().kind)))
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, Error> {
        match &self.peek().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            other => Err(self.error_here(format!("expected {what}, found {other}"))),
        }
    }

    /// A syntax error at the current token, marked "at end" if we ran out.
    fn error_here(&self, message: String) -> Error {
        let token = self.peek();
        let mut error = Error::syntax(message, token.span);
        error.at_end = token.kind == TokenKind::Eof;
        error
    }

    // ---------- statements: recursive descent ----------

    /// program → statement* EOF
    pub fn program(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        match self.peek().kind {
            TokenKind::Let => self.let_statement(),
            // `fn name(` declares a function; `fn (` starts a function value.
            TokenKind::Fn if matches!(self.tokens[self.pos + 1].kind, TokenKind::Ident(_)) => {
                self.advance();
                Ok(Stmt::Function(self.function_rest(true)?))
            }
            TokenKind::If => self.if_statement(),
            TokenKind::While => {
                self.advance();
                let cond = self.expression()?;
                let body = self.block()?;
                Ok(Stmt::While { cond, body })
            }
            TokenKind::Return => self.return_statement(),
            TokenKind::LBrace => Ok(Stmt::Block(self.block()?)),
            _ => {
                let expr = self.expression()?;
                if self.repl && self.check(&TokenKind::Eof) {
                    return Ok(Stmt::Expr(expr));
                }
                self.expect(&TokenKind::Semicolon, "';' after the expression")?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    /// let NAME (= expr)? ;
    fn let_statement(&mut self) -> Result<Stmt, Error> {
        let span = self.advance().span;
        let name = self.expect_ident("a variable name after 'let'")?;
        let init = if self.matches(&TokenKind::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon, "';' after the variable declaration")?;
        Ok(Stmt::Let { name, init, span })
    }

    /// if expr block (else (if ... | block))?
    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.advance();
        let cond = self.expression()?;
        let then_branch = self.block()?;
        let else_branch = if self.matches(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                Some(Box::new(self.if_statement()?)) // else if ...
            } else {
                Some(Box::new(Stmt::Block(self.block()?)))
            }
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    /// return expr? ;
    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let token = self.advance();
        if self.function_depth == 0 {
            return Err(Error::syntax("'return' outside a function", token.span));
        }
        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect(&TokenKind::Semicolon, "';' after the return value")?;
        Ok(Stmt::Return {
            value,
            span: token.span,
        })
    }

    /// { statement* }
    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        self.expect(&TokenKind::LBrace, "'{'")?;
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        self.expect(&TokenKind::RBrace, "'}' to close the block")?;
        Ok(statements)
    }

    /// After `fn`: NAME? ( params ) block
    fn function_rest(&mut self, named: bool) -> Result<Rc<FunctionDecl>, Error> {
        let span = self.peek().span;
        let name = if named {
            Some(self.expect_ident("a function name")?)
        } else {
            None
        };
        self.expect(&TokenKind::LParen, "'(' before the parameters")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(self.expect_ident("a parameter name")?);
                if !self.matches(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen, "')' after the parameters")?;
        self.function_depth += 1;
        let body = self.block();
        self.function_depth -= 1;
        Ok(Rc::new(FunctionDecl {
            name,
            params,
            body: body?,
            span,
        }))
    }

    // ---------- expressions: Pratt parsing ----------

    /// Parses an expression of any precedence.
    pub fn expression(&mut self) -> Result<Expr, Error> {
        self.expr_bp(0)
    }

    /// Parses an expression whose operators all bind at least `min_power`.
    fn expr_bp(&mut self, min_power: u8) -> Result<Expr, Error> {
        let mut left = self.prefix()?;

        loop {
            let token = self.peek().clone();

            // Postfix: a call. Binds tighter than everything else.
            if token.kind == TokenKind::LParen {
                if CALL_POWER < min_power {
                    break;
                }
                left = self.finish_call(left)?;
                continue;
            }

            let Some((left_power, right_power)) = infix_binding_power(&token.kind) else {
                break; // not an operator: the expression ends here
            };
            if left_power < min_power {
                break; // binds more loosely than the operator before us
            }
            self.advance();
            let right = self.expr_bp(right_power)?;
            left = self.combine(token, left, right)?;
        }
        Ok(left)
    }

    /// Things that can START an expression: literals, names, `(`, `-`, `!`, `fn`.
    fn prefix(&mut self) -> Result<Expr, Error> {
        let token = self.advance();
        let span = token.span;
        let kind = match token.kind {
            TokenKind::Number(n) => ExprKind::Number(n),
            TokenKind::Str(s) => ExprKind::Str(s.into()),
            TokenKind::True => ExprKind::Bool(true),
            TokenKind::False => ExprKind::Bool(false),
            TokenKind::Nil => ExprKind::Nil,
            TokenKind::Ident(name) => ExprKind::Variable(name),
            TokenKind::LParen => {
                let inner = self.expression()?;
                self.expect(&TokenKind::RParen, "')'")?;
                return Ok(inner); // the brackets only changed the grouping
            }
            TokenKind::Minus | TokenKind::Bang => {
                let op = if token.kind == TokenKind::Minus {
                    UnaryOp::Negate
                } else {
                    UnaryOp::Not
                };
                let operand = self.expr_bp(PREFIX_POWER)?;
                ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                }
            }
            TokenKind::Fn => ExprKind::Function(self.function_rest(false)?),
            other => {
                let mut error =
                    Error::syntax(format!("expected an expression, found {other}"), span);
                error.at_end = other == TokenKind::Eof;
                return Err(error);
            }
        };
        Ok(Expr::new(kind, span))
    }

    /// Builds the node for `left OP right`.
    fn combine(&self, op: Token, left: Expr, right: Expr) -> Result<Expr, Error> {
        let binary = match op.kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Subtract),
            TokenKind::Star => Some(BinaryOp::Multiply),
            TokenKind::Slash => Some(BinaryOp::Divide),
            TokenKind::EqualEqual => Some(BinaryOp::Equal),
            TokenKind::BangEqual => Some(BinaryOp::NotEqual),
            TokenKind::Less => Some(BinaryOp::Less),
            TokenKind::LessEqual => Some(BinaryOp::LessEqual),
            TokenKind::Greater => Some(BinaryOp::Greater),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
            _ => None,
        };
        let (left, right) = (Box::new(left), Box::new(right));
        let kind = match (binary, &op.kind) {
            (Some(bin), _) => ExprKind::Binary {
                op: bin,
                left,
                right,
            },
            (None, TokenKind::Equal) => match left.kind {
                ExprKind::Variable(name) => ExprKind::Assign { name, value: right },
                _ => return Err(Error::syntax("can only assign to a variable", op.span)),
            },
            (None, TokenKind::Or) => ExprKind::Logical {
                op: LogicalOp::Or,
                left,
                right,
            },
            (None, TokenKind::And) => ExprKind::Logical {
                op: LogicalOp::And,
                left,
                right,
            },
            _ => unreachable!("infix_binding_power only accepts the operators above"),
        };
        // Errors in `a + b` point at the operator, where the reader looks.
        Ok(Expr::new(kind, op.span))
    }

    /// After `callee`: ( args )
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let open = self.advance(); // the '('
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.expression()?);
                if !self.matches(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen, "')' after the arguments")?;
        Ok(Expr::new(
            ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            open.span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(source: &str) -> String {
        parse_expr(source).unwrap().to_string()
    }

    #[test]
    fn precedence_of_arithmetic() {
        assert_eq!(shape("1 + 2 * 3"), "(+ 1 (* 2 3))");
        assert_eq!(shape("1 * 2 + 3"), "(+ (* 1 2) 3)");
        assert_eq!(shape("(1 + 2) * 3"), "(* (+ 1 2) 3)");
        assert_eq!(shape("-2 * 3"), "(* (- 2) 3)");
        assert_eq!(shape("--x"), "(- (- x))");
    }

    #[test]
    fn left_associative_operators() {
        assert_eq!(shape("10 - 3 - 2"), "(- (- 10 3) 2)");
        assert_eq!(shape("8 / 4 / 2"), "(/ (/ 8 4) 2)");
    }

    #[test]
    fn comparison_logic_and_assignment() {
        assert_eq!(shape("1 + 2 < 4 == true"), "(== (< (+ 1 2) 4) true)");
        assert_eq!(shape("a or b and c"), "(or a (and b c))");
        assert_eq!(shape("!a == b"), "(== (! a) b)");
        assert_eq!(shape("a = b = 1 + 2"), "(= a (= b (+ 1 2)))");
    }

    #[test]
    fn calls_bind_tightest() {
        assert_eq!(shape("f(1, 2 + 3)"), "(call f 1 (+ 2 3))");
        assert_eq!(shape("-f(x)"), "(- (call f x))");
        assert_eq!(shape("make()(2)"), "(call (call make) 2)");
        assert_eq!(shape("fn (a, b) { return a; }"), "(fn (a b) ...)");
    }

    #[test]
    fn statements() {
        let program = parse(
            "let x = 1;\nfn add(a, b) { return a + b; }\nif x < 2 { x = 2; } else if x { } else { }\nwhile false { }\n{ print(x); }",
        )
        .unwrap();
        assert_eq!(program.len(), 5);
        assert!(matches!(&program[0], Stmt::Let { name, init: Some(_), .. } if name == "x"));
        let Stmt::Function(decl) = &program[1] else {
            panic!("expected a function")
        };
        assert_eq!(decl.name.as_deref(), Some("add"));
        assert_eq!(decl.params, ["a", "b"]);
        assert!(
            matches!(&program[2], Stmt::If { else_branch: Some(b), .. } if matches!(**b, Stmt::If { .. }))
        );
        assert!(matches!(&program[3], Stmt::While { .. }));
        assert!(matches!(&program[4], Stmt::Block(body) if body.len() == 1));
    }

    #[test]
    fn syntax_errors_say_what_was_expected_and_where() {
        let error = |source| parse(source).unwrap_err().to_string();
        assert_eq!(
            error("let x = 1\nprint(x);"),
            "syntax error at line 2, column 1: expected ';' after the variable declaration, found name 'print'"
        );
        assert_eq!(
            error("1 + ;"),
            "syntax error at line 1, column 5: expected an expression, found ';'"
        );
        assert_eq!(
            error("return 1;"),
            "syntax error at line 1, column 1: 'return' outside a function"
        );
        assert_eq!(
            error("1 = 2;"),
            "syntax error at line 1, column 3: can only assign to a variable"
        );
        assert_eq!(
            error("if x { print(1);"),
            "syntax error at line 1, column 17: expected '}' to close the block, found the end of the input"
        );
    }

    #[test]
    fn running_out_of_input_is_marked() {
        assert!(parse("fn f() {").unwrap_err().at_end);
        assert!(parse("1 +").unwrap_err().at_end);
        assert!(!parse("1 + ;").unwrap_err().at_end);
    }

    #[test]
    fn repl_mode_allows_a_missing_final_semicolon() {
        let tokens = lex("1 + 2").unwrap();
        assert_eq!(
            Parser::new(tokens.clone())
                .repl_mode()
                .program()
                .unwrap()
                .len(),
            1
        );
        assert!(Parser::new(tokens).program().is_err());
    }
}
