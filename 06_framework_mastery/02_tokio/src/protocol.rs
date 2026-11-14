//! The chat protocol: one line of text per message. No I/O here, so it is
//! tested with plain `#[test]`s.
//!
//! | The client sends | Meaning |
//! |---|---|
//! | `hello everyone` | say it to everyone else: `alice: hello everyone` |
//! | `/me waves`      | an action: `* alice waves` |
//! | `/who`           | list who is online (only you see the answer) |
//! | `/quit`          | leave |

/// Longest allowed name, in characters.
pub const MAX_NAME: usize = 20;

#[derive(Debug, PartialEq, Eq)]
pub enum Command<'a> {
    /// An ordinary message for everyone else.
    Say(&'a str),
    /// `/me <action>`.
    Me(&'a str),
    /// `/who`.
    Who,
    /// `/quit`.
    Quit,
    /// A line starting with `/` that isn't a known command.
    Unknown(&'a str),
    /// A blank line: ignored.
    Empty,
}

/// Turns one line from a client into a [`Command`].
pub fn parse(line: &str) -> Command<'_> {
    let line = line.trim();
    if line.is_empty() {
        return Command::Empty;
    }
    let Some(rest) = line.strip_prefix('/') else {
        return Command::Say(line);
    };
    let (command, argument) = rest.split_once(' ').unwrap_or((rest, ""));
    match (command, argument.trim()) {
        ("who", _) => Command::Who,
        ("quit", _) => Command::Quit,
        ("me", action) if !action.is_empty() => Command::Me(action),
        _ => Command::Unknown(line),
    }
}

/// A name is 1 to 20 letters, digits, `-` or `_` (after trimming).
pub fn validate_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("* a name can't be empty, try again".to_string());
    }
    if name.chars().count() > MAX_NAME {
        return Err(format!("* at most {MAX_NAME} characters, try again"));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err("* letters, digits, - and _ only, try again".to_string());
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_messages_and_commands() {
        assert_eq!(parse("  hi there "), Command::Say("hi there"));
        assert_eq!(parse("/who"), Command::Who);
        assert_eq!(parse("/quit now"), Command::Quit);
        assert_eq!(parse("/me waves  "), Command::Me("waves"));
        assert_eq!(parse("/me"), Command::Unknown("/me"));
        assert_eq!(parse("/dance"), Command::Unknown("/dance"));
        assert_eq!(parse("   "), Command::Empty);
    }

    #[test]
    fn validates_names() {
        assert_eq!(validate_name(" ada_99 "), Ok("ada_99"));
        assert!(validate_name("").is_err());
        assert!(validate_name("two words").is_err());
        assert!(validate_name(&"x".repeat(MAX_NAME + 1)).is_err());
        assert_eq!(
            validate_name(&"x".repeat(MAX_NAME)).map(str::len),
            Ok(MAX_NAME)
        );
    }
}
