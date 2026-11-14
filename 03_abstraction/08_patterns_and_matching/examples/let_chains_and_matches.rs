//! Beyond the Book: `matches!`, let chains and `if let` guards: patterns in
//! conditions.
//! Run: cargo run -p patterns_and_matching --example let_chains_and_matches

#[derive(Debug)]
enum Token {
    Number(i64),
    Word(String),
    Symbol(char),
}

fn main() {
    let tokens = vec![
        Token::Number(42),
        Token::Word("apple".into()),
        Token::Symbol('+'),
        Token::Number(-7),
        Token::Symbol('*'),
    ];

    // ---------- matches!: a pattern as a true/false test ----------
    let operators = tokens
        .iter()
        .filter(|t| matches!(t, Token::Symbol('+' | '-' | '*' | '/')))
        .count();
    let negatives = tokens
        .iter()
        .filter(|t| matches!(t, Token::Number(n) if *n < 0)) // a guard works too
        .count();
    println!("matches!:  {operators} operators, {negatives} negative number(s)");

    // ---------- let chains (2024 edition, Rust 1.88+) ----------
    // Join `let` patterns and ordinary conditions with &&. Later parts can use
    // variables bound by earlier ones. No nesting.
    let first = tokens.first();
    let second = tokens.get(1);
    if let Some(Token::Number(n)) = first
        && let Some(Token::Word(w)) = second
        && *n > 1
    {
        println!("let chain: {n} {w}s");
    }

    // The same logic without a let chain needs three levels of nesting:
    // if let Some(Token::Number(n)) = first {
    //     if let Some(Token::Word(w)) = second {
    //         if *n > 1 { ... }
    //     }
    // }

    // ---------- if let guards: a pattern inside a match guard ----------
    for token in &tokens {
        let text = match token {
            Token::Word(w) if let Some(c) = w.chars().next() => {
                format!("a word starting with {c:?}")
            }
            Token::Number(n) => format!("the number {n}"),
            other => format!("{other:?}"),
        };
        println!("if-let guard: {text}");
    }
}
