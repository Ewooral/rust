//! Stage 1: the lexer turns characters into tokens, and remembers where
//! each one started (line and column) for error messages later.
//! Run: cargo run -p capstone_interpreter --example stage1_lexer

use capstone_interpreter::lex;

fn main() {
    let source = "let total = price * 2;\nif total >= 10 { print(\"big\"); } // a comment";
    println!("source:\n{source}\n");

    let tokens = lex(source).expect("valid Kite");
    println!("{:<10} token", "position");
    for token in &tokens {
        let position = format!("{}:{}", token.span.line, token.span.col);
        println!("{position:<10} {:?}", token.kind);
    }

    // Notice what's gone: spaces, the newline and the comment. And what was
    // decided here: `>=` is ONE token, "big" lost its quotes, `let` and `if`
    // are keywords but `total` and `print` are names.

    println!("\nLexer errors stop at the first bad character:");
    for bad in ["let x = 5 $ 3;", "print(\"no end);"] {
        println!("  {bad:<18} -> {}", lex(bad).unwrap_err());
    }
}
