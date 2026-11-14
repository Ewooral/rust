//! Solution to exercise 3: make it compile (E0382), WITHOUT cloning.
//!
//! `Playlist`'s methods take `self`, so the first call MOVES the playlist
//! into the method and there is nothing left for the second call. It's
//! tempting to `#[derive(Clone)]` and call `playlist.clone().titles()`:
//! that compiles, and copies every title on every call.
//!
//! Change the two methods so they BORROW the playlist instead:
//! `titles` should return a view of the titles (`&[String]`), not a new
//! `Vec`, and `total_chars` only needs to read. Then the tests' allocation
//! check passes too: reading a playlist should cost zero allocations.
//!
//! Test: cargo test -p performance --features solutions --bin sol03_borrow_dont_clone

use performance::alloc_counter::CountingAlloc;

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

#[derive(Debug)]
struct Playlist {
    songs: Vec<String>,
}

impl Playlist {
    /// The song titles: a borrowed view, so nothing is copied.
    fn titles(&self) -> &[String] {
        &self.songs
    }

    /// How many characters all the titles have together. It only reads, so
    /// `&self` is enough.
    fn total_chars(&self) -> usize {
        self.songs.iter().map(|s| s.chars().count()).sum()
    }
}

fn sample() -> Playlist {
    Playlist {
        songs: ["Bohemian Rhapsody", "Kelele", "Blue in Green"]
            .map(String::from)
            .to_vec(),
    }
}

fn main() {
    let playlist = sample();
    println!("{} songs", playlist.titles().len());
    println!("{} characters", playlist.total_chars());
}

#[cfg(test)]
mod tests {
    use super::*;
    use performance::alloc_counter::count_allocations;

    #[test]
    fn titles_and_total_chars() {
        let playlist = sample();
        assert_eq!(playlist.titles()[1], "Kelele");
        assert_eq!(playlist.total_chars(), 17 + 6 + 13);
    }

    #[test]
    fn reading_a_playlist_allocates_nothing() {
        let playlist = sample();
        let (count, stats) = count_allocations(|| playlist.titles().len() + playlist.total_chars());
        assert_eq!(count, 3 + 36);
        assert_eq!(stats.allocations, 0, "something was copied");
    }
}
