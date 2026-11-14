//! Every file in `samples/` loads (or fails) exactly as the README says.

use std::path::Path;

use serialization::{Format, LoadError, MigrateError, ParseError, load};

fn load_sample(name: &str) -> (String, Result<serialization::Migrated, LoadError>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap();
    let format = Format::from_path(&path).unwrap();
    let result = load(&text, format);
    (text, result)
}

fn parse_error(name: &str) -> ParseError {
    match load_sample(name).1 {
        Err(LoadError::Migrate(MigrateError::Parse(err))) => err,
        other => panic!("{name}: expected a parse error, got {other:?}"),
    }
}

#[test]
fn old_versions_upgrade_and_the_current_one_loads() {
    for (name, version) in [
        ("1_v1_family_photos.json", 1),
        ("2_v2_work_laptop.json", 2),
        ("3_v3_current.toml", 3),
    ] {
        let migrated = load_sample(name).1.unwrap();
        assert_eq!(migrated.from_version, version, "{name}");
        assert_eq!(migrated.settings.version, 3, "{name}");
    }
}

#[test]
fn upgraded_settings_survive_a_round_trip_in_both_formats() {
    let settings = load_sample("2_v2_work_laptop.json").1.unwrap().settings;
    for format in [Format::Json, Format::Toml] {
        let text = format.write(&settings).unwrap();
        let again = load(&text, format).unwrap();
        assert_eq!(again.settings, settings, "{format}");
        assert!(
            again.notes.is_empty(),
            "already current: nothing to migrate"
        );
    }
}

#[test]
fn broken_files_point_at_the_right_line() {
    let err = parse_error("4_bad_syntax.json");
    assert_eq!(
        (err.line, err.column, err.message.as_str()),
        (5, 1, "trailing comma")
    );

    let err = parse_error("5_bad_type.toml");
    assert_eq!((err.line, err.column), (6, 8));
    assert_eq!(err.message, "invalid type: string \"9000\", expected u16");

    let (text, _) = load_sample("6_typo.toml");
    let err = parse_error("6_typo.toml");
    assert!(err.message.starts_with("unknown field `prot`"));
    assert!(err.snippet(&text).contains("6 | prot = 9000"));
}

#[test]
fn nonsense_values_are_all_reported() {
    let Err(LoadError::Invalid(problems)) = load_sample("7_invalid_values.toml").1 else {
        panic!("expected validation problems");
    };
    let fields: Vec<&str> = problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(
        fields,
        [
            "keep-versions",
            "server.host",
            "server.timeout",
            "folders[1].path"
        ]
    );
}

#[test]
fn files_from_the_future_are_refused() {
    assert_eq!(
        load_sample("8_too_new.toml").1,
        Err(LoadError::Migrate(MigrateError::TooNew(4)))
    );
}
