//! Solution to exercise 6: which settings does a profile really use?
//!
//! First find the starting point: the built-in defaults for `dev` and
//! `release`, or, for a custom profile, the effective settings of the
//! profile it `inherits` from (worked out the same way, recursively). Then
//! apply this profile's own table: each `Some` replaces the inherited value.
//!
//! Test: cargo test -p cargo_and_crates_io --features solutions --bin sol06_profile_inheritance

/// The settings that matter for this exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Settings {
    opt_level: u8,
    debug: bool,
    overflow_checks: bool,
}

const DEV: Settings = Settings {
    opt_level: 0,
    debug: true,
    overflow_checks: true,
};

const RELEASE: Settings = Settings {
    opt_level: 3,
    debug: false,
    overflow_checks: false,
};

/// One `[profile.NAME]` table. `None` means "not written in the table".
#[derive(Debug, Default, Clone)]
struct ProfileTable {
    name: &'static str,
    inherits: Option<&'static str>,
    opt_level: Option<u8>,
    debug: Option<bool>,
    overflow_checks: Option<bool>,
}

/// The settings `cargo build --profile <name>` would use.
fn effective(name: &str, tables: &[ProfileTable]) -> Result<Settings, String> {
    let table = tables.iter().find(|t| t.name == name);
    let base = match name {
        "dev" => DEV,
        "release" => RELEASE,
        _ => {
            let table = table.ok_or_else(|| format!("no profile named `{name}`"))?;
            let parent = table
                .inherits
                .ok_or_else(|| format!("profile `{name}` needs `inherits = \"...\"`"))?;
            effective(parent, tables)?
        }
    };
    let Some(table) = table else {
        return Ok(base);
    };
    Ok(Settings {
        opt_level: table.opt_level.unwrap_or(base.opt_level),
        debug: table.debug.unwrap_or(base.debug),
        overflow_checks: table.overflow_checks.unwrap_or(base.overflow_checks),
    })
}

fn main() {
    let tables = [ProfileTable {
        name: "release-with-debug",
        inherits: Some("release"),
        debug: Some(true),
        ..ProfileTable::default()
    }];
    for name in ["dev", "release", "release-with-debug", "turbo"] {
        println!("{name:<20} {:?}", effective(name, &tables));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_use_their_defaults() {
        assert_eq!(effective("dev", &[]), Ok(DEV));
        assert_eq!(effective("release", &[]), Ok(RELEASE));
    }

    #[test]
    fn a_table_overrides_only_what_it_writes() {
        let tables = [ProfileTable {
            name: "dev",
            opt_level: Some(1),
            ..ProfileTable::default()
        }];
        let expected = Settings {
            opt_level: 1,
            ..DEV
        };
        assert_eq!(effective("dev", &tables), Ok(expected));
    }

    #[test]
    fn custom_profiles_inherit_then_override() {
        let tables = [
            ProfileTable {
                name: "release",
                overflow_checks: Some(true),
                ..ProfileTable::default()
            },
            ProfileTable {
                name: "release-with-debug",
                inherits: Some("release"),
                debug: Some(true),
                ..ProfileTable::default()
            },
            ProfileTable {
                name: "ci",
                inherits: Some("release-with-debug"),
                opt_level: Some(2),
                ..ProfileTable::default()
            },
        ];
        let expected = Settings {
            opt_level: 3,
            debug: true,
            overflow_checks: true, // from the changed [profile.release]
        };
        assert_eq!(effective("release-with-debug", &tables), Ok(expected));
        let ci = effective("ci", &tables).unwrap();
        assert_eq!(ci.opt_level, 2);
        assert!(ci.debug, "a chain of inherits works too");
    }

    #[test]
    fn custom_profiles_need_a_table_and_inherits() {
        let missing = effective("turbo", &[]).unwrap_err();
        assert!(missing.contains("no profile"), "{missing}");
        let tables = [ProfileTable {
            name: "turbo",
            opt_level: Some(3),
            ..ProfileTable::default()
        }];
        let no_parent = effective("turbo", &tables).unwrap_err();
        assert!(no_parent.contains("inherits"), "{no_parent}");
    }
}
