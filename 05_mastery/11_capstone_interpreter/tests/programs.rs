//! Golden tests: every `programs/NAME.kite` is run through the library, and
//! what it prints must match `programs/NAME.expected` exactly.
//!
//! Adding a test is adding two files. If a change to the interpreter alters
//! an output on purpose, check the new output by eye, then update the
//! `.expected` file (`cargo run -- programs/NAME.kite > programs/NAME.expected`).
//!
//! Run: cargo test -p capstone_interpreter --test programs

use std::fs;
use std::path::PathBuf;

use capstone_interpreter::Interpreter;

#[test]
fn every_program_prints_its_expected_output() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("programs");
    let mut checked = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("programs/ exists")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "kite"))
        .collect();
    entries.sort();

    for path in entries {
        let source = fs::read_to_string(&path).unwrap();
        let expected_path = path.with_extension("expected");
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("missing {}", expected_path.display()))
            .replace("\r\n", "\n"); // in case git converted line endings

        let mut kite = Interpreter::new(Vec::new());
        // Test threads have small stacks: these programs nest calls only a
        // little, so a modest limit is plenty and can't overflow.
        kite.set_max_depth(100);
        if let Err(e) = kite.run(&source) {
            panic!("{} failed:\n{}", path.display(), e.render(&source));
        }
        let actual = String::from_utf8(kite.into_output()).unwrap();
        assert_eq!(actual, expected, "output of {}", path.display());
        checked += 1;
    }
    assert!(
        checked >= 4,
        "expected at least 4 programs, found {checked}"
    );
}
