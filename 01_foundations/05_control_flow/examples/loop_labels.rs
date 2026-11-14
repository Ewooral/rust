//! Loop labels: choose WHICH loop a `break` or `continue` applies to.
//! Run: cargo run -p control_flow --example loop_labels

fn main() {
    // The Book's example. Without a label, `break` leaves the innermost loop.
    let mut count = 0;
    'counting_up: loop {
        println!("count = {count}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break; // only the inner loop
            }
            if count == 2 {
                break 'counting_up; // the outer loop, from inside the inner one
            }
            remaining -= 1;
        }

        count += 1;
    }
    println!("End count = {count}");

    // A search: find the first seat that is free in a cinema. The label
    // stops BOTH loops as soon as we find one.
    let taken = [
        [true, true, true, true],
        [true, true, false, true],
        [false, false, false, false],
    ];
    let mut free_seat = None;
    // `.iter().enumerate()` hands out (position, item) pairs: see for_loops.
    'rows: for (row, seats) in taken.iter().enumerate() {
        for (seat, &is_taken) in seats.iter().enumerate() {
            if !is_taken {
                free_seat = Some((row, seat));
                break 'rows;
            }
        }
    }
    println!("first free seat (row, seat): {free_seat:?}");

    // `continue 'label` skips to the next round of the OUTER loop.
    'numbers: for n in 2..20u32 {
        for divisor in 2..n {
            if n.is_multiple_of(divisor) {
                continue 'numbers; // not prime: try the next n
            }
        }
        print!("{n} ");
    }
    println!("are the primes below 20");
}
