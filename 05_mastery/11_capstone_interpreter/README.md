<!-- hand-written lesson: gen_course must not overwrite -->
# Capstone: an Interpreter

**The book for this:** [Crafting Interpreters](https://craftinginterpreters.com/contents.html) by Robert Nystrom (free online; Part II is a tree-walking interpreter like this one)
· [Simple but Powerful Pratt Parsing](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) (matklad, in Rust)
· **The Rust Book:** [`Box`](https://doc.rust-lang.org/book/ch15-01-box.html) (recursive types) · [`Rc`](https://doc.rust-lang.org/book/ch15-04-rc.html) · [`RefCell`](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) · [reference cycles](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
· **std:** [`Peekable`](https://doc.rust-lang.org/std/iter/struct.Peekable.html) · [`Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html) · [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html)

You'll build an interpreter for **Kite**, a small programming language with numbers, strings, booleans, variables, arithmetic with the usual precedence, comparisons, `if`/`else`, `while`, functions with closures and recursion, and `print`. It runs script files and has a REPL. It has no dependencies at all: only the standard library.

Why build a language? Because it's the best exercise in the things Rust is famous for. An AST is a tree of **enums**, held together with **`Box`**. Evaluating it is one big **`match`**, recursing over the tree. Scopes are shared and changed from many places, which is **`Rc<RefCell<…>>`** doing real work. Errors carry positions through every stage. And every compiler error you've seen in this course came from a program like this one: rustc is a (much) bigger version.

> 💡 **The big picture: reading a recipe aloud to a robot chef.** First you **split** the recipe into words ("2", "cups", "flour"): that's the **lexer**. Then you work out the **sentence structure**: which amounts go with which ingredients, which steps are inside "repeat until golden": that's the **parser**, and the structure it builds is a family tree of instructions (the **AST**). Finally the robot **does** each instruction, keeping its ingredients on labelled shelves, with a little shelf for each step and the big pantry behind them (the **environments**). If the recipe says "add the sauce" and there is no sauce, the robot tells you exactly which line of the recipe went wrong.

## What you'll build

```text
$ cargo run -- programs/fib.kite
fib(0) = 0
fib(1) = 1
…
fib(10) = 55
fib_loop(50) = 12586269025
```

```rust
// programs/fib.kite
fn fib(n) {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
```

And a REPL that remembers what you defined, waits for you to finish an unclosed `{`, and points at your mistakes (`cargo run -p capstone_interpreter --example stage7_repl_session` shows this session):

```text
> let width = 6;
> width * 7
42
> fn area(w, h) {
...     return w * h;
... }
> area(width, 2)
12
> width / 0
runtime error at line 1, column 7: division by zero
 --> line 1, column 7
  |
1 | width / 0
  |       ^
```

## How to run it

From this folder (`05_mastery/11_capstone_interpreter`):

```bash
cargo run -- programs/fib.kite          # also: fizzbuzz.kite, closures.kite, primes.kite
cargo run                               # the REPL; quit with Ctrl+D (Ctrl+Z then Enter on Windows)
echo 'print(1 + 2 * 3);' | cargo run    # the REPL, fed by a pipe: no prompts
cargo test -p capstone_interpreter      # 56 tests: every stage, the programs, the binary
```

## Kite in one screen

| Feature | Kite | Notes |
|---|---|---|
| Values | `42`, `3.5`, `"text\n"`, `true`, `false`, `nil` | numbers are `f64`; strings escape `\n \t \" \\` |
| Variables | `let x = 1;` then `x = x + 1;` | assigning an undeclared name is an error |
| Arithmetic | `+ - * /`, unary `-`, `( )` | `*` `/` before `+` `-`; `+` also joins two strings |
| Comparison | `< <= > >= == !=` | `<` etc. on two numbers or two strings; `==` on anything |
| Logic | `and`, `or`, `!` | operands must be bools; `and`/`or` short-circuit |
| Blocks | `{ let x = 2; }` | each block is a new scope |
| `if` | `if x < 2 { … } else if … { … } else { … }` | no brackets needed; the condition must be a bool |
| `while` | `while i < 10 { … }` | |
| Functions | `fn add(a, b) { return a + b; }` | recursion works; no `return` means `nil` |
| Closures | `let f = fn (x) { return x * 2; };` | functions are values; they remember their scope |
| Built-ins | `print(a, b, …)`, `str(x)` | `print` separates with spaces; `str` makes a string |
| Comments | `// to the end of the line` | |

## The stages

| # | Stage | File | Draws on | Example |
|---|---|---|---|---|
| 1 | Tokens and spans | `token.rs`, `error.rs` | 02/01 structs, 02/02 enums | |
| 2 | The lexer | `lexer.rs` | 03/04 iterators | `stage1_lexer` |
| 3 | The AST: enums and `Box` | `ast.rs` | 03/06 `Box` | `stage2_ast_by_hand` |
| 4 | Parsing statements: recursive descent | `parser.rs` | recursion, 02/05 errors | |
| 5 | Parsing expressions: Pratt | `parser.rs` | | `stage3_pratt_parser` |
| 6 | Values | `value.rs` | 03/06 `Rc`, 03/01 traits | |
| 7 | Environments: `Rc<RefCell<Env>>` | `env.rs` | 03/06 `Rc`, `RefCell` | `stage4_environments` |
| 8 | The evaluator | `interpreter.rs` | 03/08 patterns | |
| 9 | Functions, closures and recursion | `interpreter.rs` | 03/04 closures, 03/06 cycles | `stage5_closures_and_recursion` |
| 10 | Errors with line and column | `error.rs` | | `stage6_errors` |
| 11 | The REPL and script files | `repl.rs`, `main.rs` | 05/10 CLI | `stage7_repl_session` |
| 12 | Tests for every stage | every file, `tests/` | 03/02 tests | |

## 1. Tokens and spans

A **token** is one "word" of the program: `let`, `x`, `=`, `10`, `;`. Its kind is an enum, and literal kinds carry their value:

```rust
pub enum TokenKind {
    LParen, RParen, LBrace, RBrace, Comma, Semicolon,
    Plus, Minus, Star, Slash, Bang, BangEqual, Equal, EqualEqual,
    Less, LessEqual, Greater, GreaterEqual,
    Number(f64), Str(String), Ident(String),
    Let, Fn, If, Else, While, Return, True, False, Nil, And, Or,
    Eof,
}

pub struct Token { pub kind: TokenKind, pub span: Span }
pub struct Span { pub line: usize, pub col: usize }
```

- The **span** (where the token starts) travels with it into the AST, so a *runtime* error, long after lexing, can still say "line 2, column 10".
- **`Eof`** is a real token at the end, so the parser never has to ask "are there any tokens left?"
- `TokenKind` implements `Display` the way the token looks in source (`'{'`, `name 'x'`), for error messages.

> 💡 **Analogy: words with sticky notes.** Cutting a sentence into words loses where each word was. So every word gets a sticky note: "page 2, line 10". Later, if a word turns out to be wrong, you can turn straight to the right place.

## 2. The lexer

```rust
pub fn lex(source: &str) -> Result<Vec<Token>, Error>
```

The lexer walks the characters with a `Peekable<Chars>`, which lets it **look at the next character without taking it**:

```rust
'!' if self.bump_if('=') => TokenKind::BangEqual,   // "!=" is ONE token
'!' => TokenKind::Bang,
'/' if self.bump_if('/') => { /* a comment: skip to the end of the line */ continue; }
'"' => self.string(start)?,                         // escapes: \n \t \" \\
c if c.is_ascii_digit() => self.number(c),          // 12, 3.25 (a '.' needs a digit after it)
c if c.is_alphabetic() || c == '_' => self.word(c), // a keyword, or a name
other => return Err(Error::syntax(format!("unexpected character {other:?}"), start)),
```

`bump` updates the line and column as it goes. Real output from `stage1_lexer`:

```text
position   token
1:1        Let
1:5        Ident("total")
1:11       Equal
1:13       Ident("price")
1:19       Star
1:21       Number(2.0)
1:22       Semicolon
2:1        If
…
2:10       GreaterEqual
```

Spaces, newlines and comments are gone, `>=` became one token, and `"big"` lost its quotes.

> 💡 **Analogy: cutting a sentence into word cards.** You read along, letter by letter, and snip whenever a word ends. Sometimes you peek at the next letter before snipping: after `!`, is there an `=`? Then `!=` is one card, not two. Spaces and scribbled notes in the margin (comments) go in the bin.

▶ `cargo run -p capstone_interpreter --example stage1_lexer`

## 3. The AST: enums and `Box`

The parser's output is an **abstract syntax tree**: the program's structure, with the punctuation thrown away. `1 + 2 * 3` becomes:

```text
        +
       / \
      1   *
         / \
        2   3
```

```rust
pub struct Expr { pub kind: ExprKind, pub span: Span }

pub enum ExprKind {
    Number(f64), Str(Rc<str>), Bool(bool), Nil,
    Variable(String),
    Unary   { op: UnaryOp,   operand: Box<Expr> },
    Binary  { op: BinaryOp,  left: Box<Expr>, right: Box<Expr> },
    Logical { op: LogicalOp, left: Box<Expr>, right: Box<Expr> },  // and/or: may skip `right`
    Assign  { name: String, value: Box<Expr> },
    Call    { callee: Box<Expr>, args: Vec<Expr> },
    Function(Rc<FunctionDecl>),                                    // fn (x) { … }
}
```

An expression contains expressions, so the type is **recursive**, and every link to a child goes through a pointer (`Box`, or a `Vec`, which also stores its items on the heap). Without one, the type would contain itself and have no finite size. The real error from exercise 1:

```text
error[E0072]: recursive type `Expr` has infinite size
  --> 05_mastery\11_capstone_interpreter\exercises\ex01_fix_recursive_enum.rs:16:1
   |
16 | enum Expr {
   | ^^^^^^^^^
17 |     Number(f64),
18 |     Add(Expr, Expr),
   |         ---- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
18 |     Add(Box<Expr>, Expr),
   |         ++++    +
```

Statements are a second enum, `Stmt` (`Let`, `Expr`, `Block`, `If`, `While`, `Function`, `Return`). A function's body sits in an `Rc<FunctionDecl>`, so every closure made from it shares one copy instead of cloning the tree.

`Expr` implements `Display` as an **S-expression**, operator first and fully bracketed: `(+ 1 (* 2 3))`. It's the easiest way to *see* what the parser decided, and the parser's tests compare these strings.

> 💡 **Analogy: Russian dolls with a twist.** A doll can't contain a doll exactly as big as itself, which contains another just as big, forever: that's E0072. So each doll holds a **ticket** instead (a `Box`), saying "your inner dolls are in locker 42". A ticket is always the same small size, however many dolls are behind it.

▶ `cargo run -p capstone_interpreter --example stage2_ast_by_hand` (builds the tree by hand, checks the parser builds the same one, and prints `size_of::<Expr>() = 56 bytes, size_of::<Box<Expr>>() = 8 bytes`)

## 4. Parsing statements: recursive descent

The grammar, one rule per line:

```text
program    → statement* EOF
statement  → "let" NAME ("=" expression)? ";"
           | "fn" NAME "(" params? ")" block
           | "if" expression block ("else" (if-statement | block))?
           | "while" expression block
           | "return" expression? ";"
           | block
           | expression ";"
block      → "{" statement* "}"
```

**Recursive descent** turns each rule into a method that calls the others: `statement()` looks at the first token and hands over to `let_statement()`, `if_statement()`, `block()`…, and `block()` calls `statement()` for each line inside it. The recursion in the code mirrors the nesting in the program.

```rust
fn statement(&mut self) -> Result<Stmt, Error> {
    match self.peek().kind {
        TokenKind::Let => self.let_statement(),
        TokenKind::If => self.if_statement(),
        TokenKind::While => { self.advance(); let cond = self.expression()?; let body = self.block()?; Ok(Stmt::While { cond, body }) }
        TokenKind::Return => self.return_statement(),
        TokenKind::LBrace => Ok(Stmt::Block(self.block()?)),
        // … `fn name(`, and expression statements
    }
}
```

- `expect(kind, what)` takes the token it needs or fails with **"expected `what`, found …"**, which is most of the parser's error messages for free.
- The parser counts how many functions it's inside, so `return` at the top level is a syntax error, not a runtime surprise.
- When the error is at `Eof`, it's marked `at_end`: the input stopped in the middle of something. The REPL uses that to wait for another line instead of complaining.

> 💡 **Analogy: a grammar teacher with a rule book.** For each sentence, the teacher looks at the first word. "If"? Then open the rule-book page for if-sentences: a condition, then a block, maybe an "else". A block contains sentences, so for each one the teacher goes back to the start of the rule book. Pages call pages, and the teacher always knows exactly which page they're on when something doesn't fit.

## 5. Parsing expressions: Pratt

Expressions are the hard part: `1 + 2 * 3` must group as `1 + (2 * 3)`, and `10 - 3 - 2` as `(10 - 3) - 2`. **Pratt parsing** handles all of it with one table and one loop. Every infix operator has a **binding power**, a (left, right) pair:

| Operators | Powers (left, right) | Associativity |
|---|---|---|
| `=` | (2, 1) | right: `a = b = 1` is `a = (b = 1)` |
| `or` | (3, 4) | left |
| `and` | (5, 6) | left |
| `==` `!=` | (7, 8) | left |
| `<` `<=` `>` `>=` | (9, 10) | left |
| `+` `-` | (11, 12) | left |
| `*` `/` | (13, 14) | left |
| prefix `-` `!` | 15 | |
| call `f(…)` | 17 | |

```rust
fn expr_bp(&mut self, min_power: u8) -> Result<Expr, Error> {
    let mut left = self.prefix()?;                 // a number, a name, (…), -x, !x, fn …
    loop {
        let token = self.peek().clone();
        if token.kind == TokenKind::LParen {        // postfix: a call, binds tightest
            if CALL_POWER < min_power { break; }
            left = self.finish_call(left)?;
            continue;
        }
        let Some((left_power, right_power)) = infix_binding_power(&token.kind) else {
            break;                                  // not an operator: the expression ends
        };
        if left_power < min_power {
            break;                                  // weaker than the operator before us
        }
        self.advance();
        let right = self.expr_bp(right_power)?;     // the right side: only tighter operators
        left = self.combine(token, left, right)?;
    }
    Ok(left)
}
```

The rule: **an operator only takes the expression on its left if it binds at least as tightly as the operator that came before.** The `stage3_pratt_parser` example traces a cut-down version (powers 1/2 for `+`, 3/4 for `*`). Real output:

```text
=== 1 + 2 * 3
start with 1 (min power 0)
'+' has power 1 >= 0: take it, parse its right side
  start with 2 (min power 2)
  '*' has power 3 >= 2: take it, parse its right side
    start with 3 (min power 4)
  now: (* 2 3)
now: (+ 1 (* 2 3))
result: (+ 1 (* 2 3))

=== 8 - 3 - 2
start with 8 (min power 0)
'-' has power 1 >= 0: take it, parse its right side
  start with 3 (min power 2)
  '-' has power 1 < 2: stop, give 3 back
now: (- 8 3)
…
result: (- (- 8 3) 2)
```

**Associativity is the gap between the two numbers.** For `-` (1, 2), the right side is parsed with minimum 2, so the *second* `-` (left power 1) can't join it, and the first `-` gets `3` alone: left-associative. For `=` (2, 1) it's the other way round. Exercise 3 has a parser with two wrong numbers in this table; exercise 4 adds `%`, and exercise 6 adds arrays (a prefix `[…]` and a postfix `xs[i]`).

Assignment parses like an operator, then `combine` checks the left side really is a name: `1 = 2;` is "can only assign to a variable".

> 💡 **Analogy: magnets of different strengths.** Each operator is a magnet pulling on the numbers beside it. In `1 + 2 * 3`, the `2` sits between a weak magnet (`+`) and a strong one (`*`), so the `*` wins it, and `2 * 3` sticks together first. When two magnets are equally strong (`8 - 3 - 2`), a tie-break rule says the left one wins, so you work left to right.

▶ `cargo run -p capstone_interpreter --example stage3_pratt_parser`

## 6. Values

```rust
#[derive(Clone)]
pub enum Value {
    Nil, Bool(bool), Number(f64),
    Str(Rc<str>),             // shared text: cloning copies a pointer
    Function(Rc<Closure>),    // a Kite function + the scope it was made in
    Builtin(Builtin),         // a Rust function: print, str
}
```

- **Cloning must be cheap**, because the evaluator clones values all the time (reading a variable returns a copy). `Rc<str>` and `Rc<Closure>` make a clone one pointer copy and a counter bump.
- **`PartialEq` is written by hand**: different types are never equal (`1 == "1"` is false), and functions are equal only to themselves (`Rc::ptr_eq`).
- **`Debug` is written by hand too.** A closure holds its scope, and the scope can hold the closure (a function defined in it): a derived `Debug` would print forever.
- `Display` is how `print` shows a value: Rust already prints `3.0_f64` as `3`, so whole numbers come out without `.0`.

> 💡 **Analogy: library cards, not photocopies.** When two people want the same long story (a string or a function), you don't photocopy the book: you give each a library card pointing to the one copy (`Rc`). Lending a card is instant, however long the book.

## 7. Environments: `Rc<RefCell<Env>>`

Variables live in **environments**, one per scope, each pointing at the scope around it:

```rust
pub struct Env {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}
```

- `get` looks here, then in the parent, then *its* parent, out to the globals.
- `define` (`let x = …`) always creates the variable in **this** scope, so it can **shadow** an outer `x`. `assign` (`x = …`) changes the **nearest existing** one, wherever it is.
- **Why `Rc`?** A scope has several owners: the block running in it, every child scope's `parent` link, and every closure created in it, which may outlive the block (stage 9).
- **Why `RefCell`?** `x = 10` inside a block changes a variable in a *shared* outer scope, and an `Rc` only hands out `&` references. `RefCell` moves the "one writer or many readers" check from compile time to run time. Without it, exercise 2:

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
  --> 05_mastery\11_capstone_interpreter\exercises\ex02_fix_shared_env.rs:43:29
   |
43 |             Some(parent) => parent.assign(name, value),
   |                             ^^^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Env>`
```

The price of `RefCell` is that breaking the rule **panics** at run time instead of failing to compile. So the evaluator always **evaluates first, then borrows**: `let value = self.eval(expr, env)?; env.borrow_mut().define(name, value);`. Holding `borrow_mut()` while evaluating an expression that reads the same scope would panic with "already borrowed".

> 💡 **Analogy: shelves in a kitchen.** Each step of the recipe gets its own small shelf, placed in front of the big pantry. Looking for "sugar", the robot checks the nearest shelf first, then the one behind it, then the pantry. Putting a new jar on the front shelf (`let`) can hide a jar of the same name further back (shadowing). Refilling an existing jar (`=`) refills the real one, wherever it stands. Many cooks share the pantry (`Rc`), and a rule says only one may rearrange it at a time (`RefCell`).

▶ `cargo run -p capstone_interpreter --example stage4_environments`

## 8. The evaluator

A **tree-walking** interpreter evaluates a node by evaluating its children and combining the results. It's one `match` per node type:

```rust
pub fn eval(&mut self, expr: &Expr, env: &Rc<RefCell<Env>>) -> Result<Value, Error> {
    match &expr.kind {
        ExprKind::Number(n) => Ok(Value::Number(*n)),
        ExprKind::Variable(name) => env.borrow().get(name).ok_or_else(|| {
            Error::runtime(format!("undefined variable '{name}'"), expr.span)
        }),
        ExprKind::Binary { op, left, right } => {
            let left = self.eval(left, env)?;
            let right = self.eval(right, env)?;
            binary(*op, left, right).map_err(|message| Error::runtime(message, expr.span))
        }
        // … Unary, Logical, Assign, Call, Function
    }
}
```

- **`binary`** is a `match` on `(op, &left, &right)`: `(Add, Number(a), Number(b))`, `(Add, Str(a), Str(b))`, `(Divide, Number(_), Number(b)) if *b == 0.0` → "division by zero", and a final arm for every type mismatch: "cannot use '+' on a number and a string". Pattern matching on tuples ([03/08](../../03_abstraction/08_patterns_and_matching/README.md)) is what makes this readable.
- **`and`/`or` short-circuit:** `false and anything` never evaluates `anything`. That's why they're a separate `Logical` node, not a `Binary` one (which evaluates both sides first).
- **Conditions must be bools.** `if 1 { }` is a runtime error, rather than guessing what "truthy" should mean.
- **`return` isn't an error, it's control flow.** Statements return `Flow::Normal` or `Flow::Return(value)`, and every block passes a `Return` straight up until a function call catches it. (Some interpreters abuse `Err` for this; a separate type keeps errors meaning errors.)
- **Output goes to a `W: Write`**, given to `Interpreter::new`. The binary passes stdout, and the tests pass a `Vec<u8>` and read it back: the same trick as lesson 05/10's renderers.
- **Built-ins** are plain `fn` pointers, `fn(&[Value], &mut dyn Write) -> Result<Value, String>`, registered with `define_builtin`. The interpreter adds the name and position to their errors (exercise 5 adds `len`).

> 💡 **Analogy: the robot chef cooks the tree from the leaves up.** To do "mix (flour + sugar) with eggs", it first fetches flour and sugar, combines them, fetches the eggs, then mixes. Each step asks its smaller steps first. "Add flour to the music" isn't a recipe it can follow, so it stops and says which line said that.

## 9. Functions, closures and recursion

A function value is a **closure**: the function's code plus the environment it was **created** in.

```rust
pub struct Closure {
    pub decl: Rc<FunctionDecl>,
    pub env: Rc<RefCell<Env>>,
}
```

Calling one (the heart of `call`):

```rust
let scope = Env::child(&closure.env);        // parent: where it was DEFINED, not where it's called
for (param, arg) in decl.params.iter().zip(args) {
    scope.borrow_mut().define(param.clone(), arg);
}
let flow = self.exec_block(&decl.body, scope);
match flow? {
    Flow::Return(value) => Ok(value),
    Flow::Normal => Ok(Value::Nil),
}
```

- **Closures** fall out of that first line. `make_counter` returns an inner function whose `env` is the call's scope, so `count` lives on after `make_counter` has returned, and each call to `make_counter` makes a separate `count`.
- **Lexical scope:** a function sees the variables where it was *written*. In `stage5_closures_and_recursion`, `show_who()` called from inside `caller()` still prints "global", not the caller's own `who`.
- **Recursion** works because `fn fib` defines `fib` in the scope the closure captured, so by the time `fib` runs, the name is there.
- **The cost: a reference cycle.** That scope holds `fib`, and `fib`'s closure holds the scope: two `Rc`s pointing at each other are never freed ([03/06, reference cycles](../../03_abstraction/06_smart_pointers/README.md#6-reference-cycles-can-leak-memory)). For a script that runs and exits, it doesn't matter. Long-running language runtimes use a **garbage collector** instead, which can find and free cycles.
- **Arity** is checked: "f takes 1 argument but was given 2".

**Recursion depth.** Each Kite call is several nested Rust calls (`call` → `exec_block` → `exec` → `eval` → …), and debug builds use big stack frames. Measured on this lesson's debug build: 50 nested Kite calls fit in a 1 MiB stack, 100 overflowed it, and 999 fit in 16 MiB. A stack overflow aborts the whole process. So:

- the interpreter counts calls and stops at a limit (1000 by default, `set_max_depth` to change it) with "too much recursion";
- `main` runs everything on a thread with a **64 MiB** stack (`thread::Builder::new().stack_size(…)`), because Windows' main thread has 1 MiB. The operating system only reserves that address space; memory is used as the stack actually grows;
- the library's tests, which run on the test harness's smaller threads, lower the limit.

> 💡 **Analogy: a recipe card with a lunchbox.** When you write a recipe card inside someone's kitchen, you pack a lunchbox with what's on their shelves and clip it to the card (the closure). Wherever the card is used later, even after that kitchen has closed, the cook opens the lunchbox. And a recipe can say "now do this same recipe again with half the dough" (recursion), as long as someone counts the levels so nobody ends up folding pastry forever.

▶ `cargo run -p capstone_interpreter --example stage5_closures_and_recursion`

## 10. Errors with line and column

Every stage reports problems with one type, `Error { kind: Syntax | Runtime, message, span, at_end }`, and `render` prints it the way rustc prints yours. Real output from `stage6_errors`:

```text
--- found by the parser:
syntax error at line 3, column 1: expected ';' after the return value, found '}'
 --> line 3, column 1
  |
3 | }
  | ^

--- found by the runtime:
runtime error at line 2, column 22: division by zero
 --> line 2, column 22
  |
2 | print("average:", 10 / items);
  |                      ^

--- found by the runtime:
runtime error at line 2, column 6: greet takes 1 argument but was given 0
 --> line 2, column 6
  |
2 | greet();
  |      ^
```

- Spans point where a reader looks: binary errors at the **operator**, call errors at the **`(`**, an undefined variable at the **name**.
- The missing `;` is reported at the `}` on the next line: that's where the parser found something it didn't expect. A friendlier parser could point just after `w * h`, where the `;` belongs.
- The caret's indentation copies tabs from the source line, so it still lines up in tab-indented code.

> 💡 **Analogy: a teacher's red pen.** "Wrong" on its own is useless. A good teacher circles the exact word, writes the line number, and says what was expected there.

▶ `cargo run -p capstone_interpreter --example stage6_errors`

## 11. The REPL and script files

The REPL logic is a library type with no terminal in it, so tests can drive it with strings:

```rust
pub enum Event { NeedMore, Done(Option<Value>), Error(String) }

impl<W: Write> Repl<W> {
    pub fn feed(&mut self, line: &str) -> Event
}
```

- An **expression** shows its value (`width * 7` → `42`); a **statement** shows nothing. In the REPL, the last expression may leave out its `;` (the parser's `repl_mode`).
- If the error is at the end of the input (`fn area(w, h) {`, `(1 +`), it returns `NeedMore`, keeps the text, and the next line is added to it. A blank line gives up and shows the error.
- Variables and functions persist between entries: one `Interpreter`, many `run_repl` calls.

`main.rs` adds the outside world, using lesson [05/10](../10_capstone_cli_tool/README.md)'s habits: prompts and the banner only when stdin **is a terminal** (so `echo … | kite` prints just results), errors on **stderr**, and exit codes:

| Code | Meaning |
|---|---|
| 0 | ok |
| 64 | usage: more than one argument |
| 65 | syntax error in the script |
| 66 | the script file can't be read |
| 70 | runtime error (or a piped REPL session that hit one) |

These come from the Unix `sysexits.h` conventions; Crafting Interpreters uses 64, 65 and 70 the same way.

> 💡 **Analogy: a conversation versus a letter.** A script file is a letter: the robot reads the whole thing and then cooks. The REPL is a conversation: you say one line, it answers, and if you stop mid-sentence ("and then mix…"), it waits politely for the rest instead of saying "pardon?".

▶ `cargo run -p capstone_interpreter --example stage7_repl_session`, then the real thing: `cargo run -p capstone_interpreter`

## 12. Tests for every stage

| Where | What it checks | Count |
|---|---|---|
| `src/lexer.rs` | tokens, two-character operators, strings, spans, lexer errors | 7 |
| `src/parser.rs` | precedence and associativity (as S-expressions), statements, syntax errors, `at_end` | 8 |
| `src/env.rs`, `src/value.rs`, `src/error.rs` | scopes and shadowing, equality, rendering | 9 |
| `src/interpreter.rs` | arithmetic, strings, logic, scopes, functions, closures, recursion, runtime errors | 13 |
| `src/repl.rs` | events, continuation lines, errors | 4 |
| `tests/programs.rs` | **golden tests**: each `programs/*.kite` must print exactly its `.expected` file | 1 (4 programs) |
| `tests/cli.rs` | the real binary: scripts, a piped REPL, exit codes, deep recursion on the big stack | 8 |
| doc comments | the examples in the API docs | 6 |

```text
$ cargo test -p capstone_interpreter
test result: ok. 41 passed …    (unit tests, src/)
test result: ok. 8 passed …     (tests/cli.rs)
test result: ok. 1 passed …     (tests/programs.rs)
test result: ok. 6 passed …     (doc tests)
```

**Golden tests** scale well for a language: adding a test is adding two files, `programs/NAME.kite` and `programs/NAME.expected`. When you change the interpreter on purpose, check the new output by eye and regenerate the file (`cargo run -- programs/NAME.kite > programs/NAME.expected`).

> 💡 **Analogy: checking every station on the production line.** Inspect the word-cutter, the grammar teacher and the chef separately (unit tests), then taste the finished dishes against photos of how they should look (golden tests), then order from the front counter like a real customer (the binary tests).

## Where real interpreters go next

- **A resolver pass** works out, before running, which scope each name refers to, so lookups don't walk the chain ([Crafting Interpreters ch. 11](https://craftinginterpreters.com/resolving-and-binding.html)).
- **Bytecode:** compile the AST to a flat list of simple instructions for a loop to run (Part III of the book). Much faster, and no deep Rust recursion.
- **A garbage collector** instead of `Rc`, which also frees cycles.
- **More language:** `%`, arrays and `len` (the exercises), `for` loops, `break`, maps, classes.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `enum Expr { Add(Expr, Expr), … }` | E0072 recursive type has infinite size | `Box<Expr>` for each child |
| `parent: Option<Rc<Env>>` and then `parent.assign(…)` | E0596 cannot borrow data in an `Rc` as mutable | `Rc<RefCell<Env>>` and `.borrow_mut()` |
| `env.borrow_mut()` held while evaluating | panic: already borrowed | Evaluate first, then borrow. |
| `'*' => (3, 4)` below `'+' => (5, 6)` | `1 + 2 * 3` gives 9 | Tighter operators get higher powers. |
| `'-' => (5, 4)` | `10 - 3 - 2` gives 9 | Left-associative: right power = left + 1. |
| A call's new scope made from the *caller's* env | closures see the wrong variables (dynamic scope) | `Env::child(&closure.env)` |
| `#[derive(Debug)]` on values holding closures | `{:?}` recurses through the scope cycle | Write `Debug` by hand. |
| `return` signalled with `Err(…)` | real errors and returns get mixed up | A `Flow` enum. |
| Evaluating both sides of `and`/`or` | `false and crash()` crashes | A separate `Logical` node that may skip `right`. |
| Unbounded Kite recursion | Rust stack overflow aborts the process | A depth limit, and a bigger thread stack. |
| `1.` in Kite source | syntax error: unexpected character '.' | Write `1` or `1.0`. |
| Errors without spans | "division by zero" (but where?) | Carry `Span` from tokens into the AST. |
| Comparing `.expected` files byte for byte on Windows | tests fail on `\r\n` after a git checkout | Normalise `\r\n` to `\n` first. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p capstone_interpreter --example stage1_lexer
cargo run -p capstone_interpreter --example stage2_ast_by_hand
cargo run -p capstone_interpreter --example stage3_pratt_parser
cargo run -p capstone_interpreter --example stage4_environments
cargo run -p capstone_interpreter --example stage5_closures_and_recursion
cargo run -p capstone_interpreter --example stage6_errors
cargo run -p capstone_interpreter --example stage7_repl_session
```

**2. Do the exercises** in `exercises/`. Numbers 3 to 6 extend the language:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_recursive_enum` | Make it compile (E0072) with `Box` | `cargo run -p capstone_interpreter --features exercises --bin ex01_fix_recursive_enum` |
| 2 | `ex02_fix_shared_env` | Make it compile (E0596) with `Rc<RefCell<…>>` | `cargo run -p capstone_interpreter --features exercises --bin ex02_fix_shared_env` |
| 3 | `ex03_fix_precedence` | Fix two binding powers, pass 3 tests | `cargo test -p capstone_interpreter --features exercises --bin ex03_fix_precedence` |
| 4 | `ex04_add_modulo` | Add `%` in the lexer, parser and evaluator, pass 4 tests | `cargo test -p capstone_interpreter --features exercises --bin ex04_add_modulo` |
| 5 | `ex05_len_builtin` | A `len` built-in for the real Kite, pass 3 tests | `cargo test -p capstone_interpreter --features exercises --bin ex05_len_builtin` |
| 6 | `ex06_add_arrays` | Array literals and indexing (prefix and postfix), pass 4 tests | `cargo test -p capstone_interpreter --features exercises --bin ex06_add_arrays` |

Exercises 3, 4 and 6 use a small standalone calculator built exactly like Kite's expression parser, so each change is small and visible. Exercise 5 extends the real interpreter through its public API.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p capstone_interpreter --features solutions --bins`, and `cargo run -p capstone_interpreter --features solutions --bin sol01_fix_recursive_enum`, etc.).

**4. The project: extend the real Kite.** Each feature touches the stages in order, and the compiler helps: add a variant to `TokenKind` or `BinaryOp`, and every `match` that's now missing a case is an error until you handle it.
- **`%`** (as in exercise 4): `TokenKind::Percent` in `token.rs` and `lexer.rs`, a binding power and a `BinaryOp::Remainder` in `parser.rs`/`ast.rs`, an arm in `binary` in `interpreter.rs`. Then rewrite `programs/fizzbuzz.kite` with `n % 15 == 0`, regenerate its `.expected` file, and check the output didn't change.
- **`len`** (exercise 5) as a permanent built-in next to `print` and `str`.
- **Arrays** (exercise 6): a `Value::Array(Rc<RefCell<Vec<Value>>>)` (why `RefCell`, if arrays can be changed?), `[a, b]`, `xs[i]`, and `len(xs)`.
- **`break`**: a new `Flow::Break`, which `while` catches and functions refuse.
- Write a Kite program of your own (a guessing game with a fixed "secret", or a times table) and add it to `programs/` with its `.expected` output.

## Checklist

- [ ] A lexer: `Peekable<Chars>`, lookahead for two-character operators, line and column tracking
- [ ] Tokens with spans, and spans carried into the AST
- [ ] A recursive AST of enums with `Box`, and why E0072 needs it
- [ ] Recursive descent for statements, with "expected … found …" errors
- [ ] Pratt parsing: binding powers, associativity, prefix and postfix operators
- [ ] Cheap-to-clone values with `Rc`, hand-written `PartialEq` and `Debug`
- [ ] Scopes as `Rc<RefCell<Env>>`: define, shadow, assign, and evaluate-then-borrow
- [ ] A tree-walking evaluator: short-circuit logic, a `Flow` enum for `return`, runtime errors with positions
- [ ] Closures capturing their defining scope, recursion, the `Rc` cycle it creates, and a recursion limit
- [ ] A REPL that waits for unfinished input, and a script runner with exit codes
- [ ] Unit tests per stage, golden tests for programs, and tests of the real binary
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- An interpreter is a pipeline: the **lexer** turns text into tokens, the **parser** turns tokens into an **AST**, and the **evaluator** walks the AST to produce values and output.
- Recursive data needs indirection: `Box` in the AST, `Rc` for anything shared (strings, function bodies, scopes), `RefCell` for shared things that change.
- Recursive descent gives one method per grammar rule; Pratt parsing gives precedence and associativity from a table of (left, right) binding powers.
- A closure is code plus the environment it was created in; calling it makes a new scope whose parent is that environment.
- Positions travel from tokens to the AST to runtime errors, so every error can point at the exact line and column.

**In one sentence for a child:** an interpreter is a robot chef that cuts your recipe into words, works out how the sentences fit together, and then cooks it step by step, telling you exactly which line was wrong if it can't.

**Next:** [06 Framework Mastery: Axum](../../06_framework_mastery/01_axum/README.md). Framework Mastery begins: from here on, the course builds with real-world crates.
