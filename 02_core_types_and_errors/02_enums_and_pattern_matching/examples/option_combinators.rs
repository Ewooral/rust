//! Beyond the Book: working with Option without writing a match every time.
//! Run: cargo run -p enums_and_pattern_matching --example option_combinators

#[derive(Debug)]
struct User {
    name: String,
    nickname: Option<String>,
    age: Option<u8>,
}

/// `?` on an Option: if it is None, return None from this function right
/// away; if it is Some(v), carry on with v.
fn nickname_length(users: &[User], name: &str) -> Option<usize> {
    let user = users.iter().find(|u| u.name == name)?;
    let nick = user.nickname.as_ref()?;
    Some(nick.len())
}

fn main() {
    let users = vec![
        User {
            name: "Ama".into(),
            nickname: Some("Ams".into()),
            age: Some(9),
        },
        User {
            name: "Kofi".into(),
            nickname: None,
            age: None,
        },
    ];

    // map: change the value inside Some, leave None alone.
    let age_next_year = users[0].age.map(|a| a + 1);
    println!("map: {age_next_year:?}");

    // unwrap_or / unwrap_or_else / unwrap_or_default: a fallback for None.
    // (`unwrap_or_else` only builds the fallback when it is needed.)
    let kofi_age = users[1].age.unwrap_or(0);
    let shown = users[1]
        .nickname
        .clone()
        .unwrap_or_else(|| users[1].name.clone());
    println!("unwrap_or: {kofi_age}, unwrap_or_else: {shown}");

    // and_then: chain steps that can each fail (each returns an Option).
    let ama_nick = users
        .iter()
        .find(|u| u.name == "Ama")
        .and_then(|u| u.nickname.as_deref());
    println!("and_then: {ama_nick:?}");

    // filter: keep Some only if the test passes.
    let child = users[0].age.filter(|a| *a < 13);
    println!("filter: {child:?}");

    // ok_or: turn an Option into a Result with an error message (lesson 02/05).
    let r: Result<u8, &str> = users[1].age.ok_or("age unknown");
    println!("ok_or: {r:?}");

    // `?` keeps a chain of lookups flat.
    println!("? : {:?}", nickname_length(&users, "Ama"));
    println!("? : {:?}", nickname_length(&users, "Kofi"));
    println!("? : {:?}", nickname_length(&users, "Yaw"));

    // matches!: a quick true/false pattern test. `if` adds an extra
    // condition to the pattern (a "match guard", Book chapter 19).
    let old_enough = users
        .iter()
        .filter(|u| matches!(u.age, Some(a) if a >= 9))
        .count();
    println!("matches!: {old_enough} user(s) are 9 or older");
    // For a plain "is it Some?" use `.is_some()` / `.is_none()`.
    println!("is_some: {}", users[1].nickname.is_some());
}
