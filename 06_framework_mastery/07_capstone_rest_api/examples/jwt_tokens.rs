//! JSON Web Tokens: what is inside one, and what makes the server refuse it.
//! Run: cargo run -p capstone_rest_api --example jwt_tokens

use std::time::Duration;

use capstone_rest_api::auth::Tokens;
use capstone_rest_api::config::Secret;
use capstone_rest_api::domain::Role;

/// Decodes one base64url part, to show that anyone can READ a token.
fn base64url_decode(part: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for byte in part.bytes() {
        let Some(value) = ALPHABET.iter().position(|&c| c == byte) else {
            continue;
        };
        bits = (bits << 6) | value as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The other direction, for the forgery below.
fn base64url_encode(text: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in text.as_bytes().chunks(3) {
        let n = chunk.iter().fold(0u32, |n, &b| (n << 8) | u32::from(b)) << (8 * (3 - chunk.len()));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

fn main() {
    let secret = Secret::new("example-secret-example-secret-1234");
    let tokens =
        Tokens::new(&secret, "capstone-rest-api", Duration::from_secs(3600)).expect("valid keys");

    let token = tokens.issue(42, Role::Member).expect("signing works");
    println!("a token ({} characters):\n  {token}\n", token.len());
    let parts: Vec<&str> = token.split('.').collect();
    println!("header:    {}", base64url_decode(parts[0]));
    println!("claims:    {}", base64url_decode(parts[1]));
    println!(
        "signature: {} (HMAC-SHA256 of the first two parts)\n",
        parts[2]
    );

    println!(
        "verify(valid)       -> {:?}",
        tokens.verify(&token).map(|c| c.sub)
    );

    // Promote yourself: rewrite the claims to say "admin", keep the old
    // signature. Reading was easy; forging isn't: the signature no longer matches.
    let promoted = base64url_decode(parts[1]).replace("member", "admin");
    let forged = format!("{}.{}.{}", parts[0], base64url_encode(&promoted), parts[2]);
    println!("forged claims: {promoted}");
    println!(
        "verify(edited)      -> {:?}",
        tokens.verify(&forged).map(|c| c.sub)
    );

    let other = Tokens::new(
        &Secret::new("a-different-secret-a-different-123"),
        "capstone-rest-api",
        Duration::from_secs(3600),
    )
    .expect("valid keys");
    println!(
        "verify(other key)   -> {:?}",
        other.verify(&token).map(|c| c.sub)
    );

    let yesterday = jsonwebtoken::get_current_timestamp() - 24 * 3600;
    let old = tokens
        .issue_at(42, Role::Member, yesterday)
        .expect("signing works");
    println!(
        "verify(expired)     -> {:?}",
        tokens.verify(&old).map(|c| c.sub)
    );
}
