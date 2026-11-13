//! Catch-all patterns: `other` (binds the value) and `_` (ignores it).
//! Run: cargo run -p enums_and_pattern_matching --example catch_all

fn add_fancy_hat() {
    println!("  you get a fancy hat");
}
fn remove_fancy_hat() {
    println!("  you lose your fancy hat");
}
fn move_player(spaces: u8) {
    println!("  move {spaces} spaces");
}
fn reroll() {
    println!("  roll again");
}

fn main() {
    for dice_roll in [3, 7, 9] {
        println!("rolled {dice_roll}, version 1 (use the value):");
        match dice_roll {
            3 => add_fancy_hat(),
            7 => remove_fancy_hat(),
            other => move_player(other), // a variable pattern matches anything
        }

        println!("rolled {dice_roll}, version 2 (ignore the value):");
        match dice_roll {
            3 => add_fancy_hat(),
            7 => remove_fancy_hat(),
            _ => reroll(), // `_` matches anything and does not bind it
        }

        println!("rolled {dice_roll}, version 3 (do nothing):");
        match dice_roll {
            3 => add_fancy_hat(),
            7 => remove_fancy_hat(),
            _ => (), // the unit value: explicitly "nothing happens"
        }
    }

    // The catch-all must come LAST. Arms are tried in order, so anything
    // after it can never run (the compiler warns: unreachable pattern).
}
