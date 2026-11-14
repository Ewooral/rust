//! Snapshot testing with insta: compare big outputs with a saved, reviewed copy.
//!
//! Run:  cargo run -p advanced_testing --example snapshot_testing
//! Test: cargo test -p advanced_testing --example snapshot_testing
//!
//! The saved snapshots are in examples/snapshots/. Change the table below
//! (say, a column width) and run the tests again to see a snapshot diff.

#[derive(Debug)]
struct Player {
    name: &'static str,
    played: u32,
    won: u32,
}

/// A league table, as text. Exactly the kind of output that is tedious to
/// check with assert_eq! line by line, and easy to break by accident.
fn league_table(players: &[Player]) -> String {
    let mut sorted: Vec<&Player> = players.iter().collect();
    sorted.sort_by(|a, b| b.won.cmp(&a.won).then(a.name.cmp(b.name)));
    let mut out = format!(
        "{:<4}{:<10}{:>4}{:>4}{:>5}\n",
        "#", "Player", "P", "W", "Pts"
    );
    for (i, p) in sorted.iter().enumerate() {
        out += &format!(
            "{:<4}{:<10}{:>4}{:>4}{:>5}\n",
            i + 1,
            p.name,
            p.played,
            p.won,
            p.won * 3
        );
    }
    out
}

fn players() -> Vec<Player> {
    vec![
        Player {
            name: "Kofi",
            played: 5,
            won: 2,
        },
        Player {
            name: "Ama",
            played: 5,
            won: 4,
        },
        Player {
            name: "Esi",
            played: 4,
            won: 2,
        },
    ]
}

fn main() {
    print!("{}", league_table(&players()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, assert_snapshot};

    // Saved to examples/snapshots/snapshot_testing__tests__league_table.snap
    #[test]
    fn league_table() {
        assert_snapshot!(super::league_table(&players()));
    }

    // {:#?} of a value: good for whole structs.
    #[test]
    fn players_as_debug() {
        assert_debug_snapshot!(players()[0]);
    }

    // Inline: the expected text lives in the source, after the @.
    #[test]
    fn empty_league_has_only_the_header() {
        assert_snapshot!(super::league_table(&[]), @"#   Player       P   W  Pts");
    }
}
