//! Solution to exercise 6: read old files and upgrade them (pass 4 tests).
//!
//! A pet clinic's app has saved pets in two formats over the years:
//!
//!     version 1 (no "version" field):
//!         {"name": "Rex", "kind": "dog", "age_years": 3}
//!     version 2 (the current one):
//!         {"version": 2, "name": "Rex", "species": "Dog", "born": 2023}
//!
//! Implement the three functions, using the recipe from the lesson:
//!   - `version_of`: read ONLY the version with the `Probe` struct. A missing
//!     version means 1. (Why must `Probe` NOT have deny_unknown_fields?
//!     Answer: it reads whole pet files but only knows `version`; with
//!     deny_unknown_fields, every real file would be an error.)
//!   - `upgrade`: PetV1 -> PetV2. "dog", "cat" and "rabbit" (in any case)
//!     become a Species; anything else is Err("unknown kind: <kind>").
//!     `born` is THIS_YEAR - age_years.
//!   - `load`: probe, then parse the text as the right version's struct
//!     (and upgrade v1). Any other version is Err("unsupported version: N").
//!     Turn serde_json errors into Strings with `.to_string()`.
//!
//! Test: cargo test -p serialization --features solutions --bin sol06_migrate_versions

use serde::{Deserialize, Serialize};

const THIS_YEAR: u16 = 2026;

#[derive(Debug, Deserialize)]
struct Probe {
    version: Option<u32>,
}

/// Version 1: frozen. Read, never written.
#[derive(Debug, Deserialize)]
struct PetV1 {
    name: String,
    kind: String,
    age_years: u16,
}

/// Version 2: the current format.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PetV2 {
    version: u32,
    name: String,
    species: Species,
    born: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Species {
    Dog,
    Cat,
    Rabbit,
}

fn version_of(json: &str) -> Result<u32, serde_json::Error> {
    let probe: Probe = serde_json::from_str(json)?;
    Ok(probe.version.unwrap_or(1))
}

fn upgrade(old: PetV1) -> Result<PetV2, String> {
    let species = match old.kind.to_lowercase().as_str() {
        "dog" => Species::Dog,
        "cat" => Species::Cat,
        "rabbit" => Species::Rabbit,
        _ => return Err(format!("unknown kind: {}", old.kind)),
    };
    Ok(PetV2 {
        version: 2,
        name: old.name,
        species,
        born: THIS_YEAR.saturating_sub(old.age_years),
    })
}

fn load(json: &str) -> Result<PetV2, String> {
    match version_of(json).map_err(|err| err.to_string())? {
        1 => {
            let old: PetV1 = serde_json::from_str(json).map_err(|err| err.to_string())?;
            upgrade(old)
        }
        2 => serde_json::from_str(json).map_err(|err| err.to_string()),
        other => Err(format!("unsupported version: {other}")),
    }
}

fn main() {
    for json in [
        r#"{"name": "Rex", "kind": "dog", "age_years": 3}"#,
        r#"{"version": 2, "name": "Mia", "species": "Cat", "born": 2020}"#,
        r#"{"version": 3, "name": "Zoom", "species": "Robot"}"#,
    ] {
        println!("{json}\n  -> {:?}", load(json));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probes_the_version() {
        let v1 = r#"{"name": "Rex", "kind": "dog", "age_years": 3}"#;
        assert_eq!(version_of(v1).unwrap(), 1);
        assert_eq!(version_of(r#"{"version": 2, "anything": [1]}"#).unwrap(), 2);
        assert!(version_of("not json").is_err());
    }

    #[test]
    fn upgrades_version_1() {
        let pet = load(r#"{"name": "Rex", "kind": "Dog", "age_years": 3}"#).unwrap();
        assert_eq!(
            pet,
            PetV2 {
                version: 2,
                name: "Rex".to_string(),
                species: Species::Dog,
                born: 2023
            }
        );
        let err = load(r#"{"name": "Nemo", "kind": "fish", "age_years": 1}"#).unwrap_err();
        assert_eq!(err, "unknown kind: fish");
    }

    #[test]
    fn reads_version_2_as_it_is() {
        let json = r#"{"version": 2, "name": "Mia", "species": "Cat", "born": 2020}"#;
        let pet = load(json).unwrap();
        assert_eq!(pet.species, Species::Cat);
        assert_eq!(pet.born, 2020);
    }

    #[test]
    fn refuses_unknown_versions_and_bad_files() {
        assert_eq!(
            load(r#"{"version": 3, "name": "Zoom"}"#).unwrap_err(),
            "unsupported version: 3"
        );
        let json = r#"{"version": 2, "name": "Mia", "species": "Cat", "born": 2020, "x": 1}"#;
        let err = load(json).unwrap_err();
        assert!(err.starts_with("unknown field `x`"), "{err}");
    }
}
