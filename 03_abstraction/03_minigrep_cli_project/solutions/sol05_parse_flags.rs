//! Solution to exercise 5: command-line FLAGS, not just positional arguments.
//!
//! Implement `Config::build` so it accepts, in any order:
//!   -i or --ignore-case    case-insensitive search
//!   -n or --line-number    show line numbers
//!   exactly two other arguments: the query, then the file path
//! Errors (as `Err` messages, never panics):
//!   an unknown flag (anything else starting with '-')  -> "unknown flag: -x"
//!   fewer than two positional arguments                -> "not enough arguments"
//!   more than two                                      -> "too many arguments"
//! Remember args[0] is the program name: skip it.
//!
//! Test: cargo test -p minigrep --features solutions --bin sol05_parse_flags

#[derive(Debug, PartialEq)]
struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
    line_numbers: bool,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, String> {
        let mut ignore_case = false;
        let mut line_numbers = false;
        let mut positional = Vec::new();

        for arg in args.iter().skip(1) {
            match arg.as_str() {
                "-i" | "--ignore-case" => ignore_case = true,
                "-n" | "--line-number" => line_numbers = true,
                flag if flag.starts_with('-') => return Err(format!("unknown flag: {flag}")),
                value => positional.push(value.to_string()),
            }
        }

        match positional.len() {
            0 | 1 => Err("not enough arguments".to_string()),
            2 => {
                let file_path = positional.pop().expect("two items");
                let query = positional.pop().expect("one item");
                Ok(Config {
                    query,
                    file_path,
                    ignore_case,
                    line_numbers,
                })
            }
            _ => Err("too many arguments".to_string()),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match Config::build(&args) {
        Ok(config) => println!("{config:?}"),
        Err(e) => eprintln!("error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        std::iter::once("minigrep")
            .chain(list.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn plain_positional_arguments() {
        assert_eq!(
            Config::build(&args(&["to", "poem.txt"])),
            Ok(Config {
                query: "to".into(),
                file_path: "poem.txt".into(),
                ignore_case: false,
                line_numbers: false,
            })
        );
    }

    #[test]
    fn flags_anywhere_short_or_long() {
        let config = Config::build(&args(&["-n", "to", "--ignore-case", "poem.txt"])).unwrap();
        assert!(config.ignore_case && config.line_numbers);
        assert_eq!(
            (config.query.as_str(), config.file_path.as_str()),
            ("to", "poem.txt")
        );
        let config = Config::build(&args(&["to", "poem.txt", "-i", "--line-number"])).unwrap();
        assert!(config.ignore_case && config.line_numbers);
    }

    #[test]
    fn errors() {
        assert_eq!(
            Config::build(&args(&["-x", "to", "poem.txt"])),
            Err("unknown flag: -x".to_string())
        );
        assert_eq!(
            Config::build(&args(&["to"])),
            Err("not enough arguments".to_string())
        );
        assert_eq!(
            Config::build(&args(&["a", "b", "c"])),
            Err("too many arguments".to_string())
        );
    }
}
