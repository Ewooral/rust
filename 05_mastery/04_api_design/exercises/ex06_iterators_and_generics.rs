//! Exercise 6: flexible arguments and iterator-returning methods (pass 4 tests).
//!
//! Implement each `todo!()` in `Playlist`:
//!
//!   add(title, seconds)   `impl Into<String>`, returns `&mut Self` so calls
//!                         chain: `list.add("a", 60).add("b", 90);`
//!   iter()                the songs, borrowed (a named type: `slice::Iter`)
//!   titles()              just the titles, as `&str`
//!   longer_than(s)        the songs strictly longer than `s` seconds
//!   total_seconds()       the sum of all lengths
//!   find(words)           the first song whose title contains `words`,
//!                         ignoring letter case. `impl AsRef<str>`, so a
//!                         `&str` or a `String` both work.
//!   IntoIterator for &Playlist   so `for song in &list` works
//!   FromIterator<(S, u32)>       so `[("a", 60), ("b", 90)].into_iter().collect()`
//!                                makes a Playlist, for any `S: Into<String>`
//!
//! The methods that return `impl Iterator` start as `{ todo!(); empty() }`:
//! `todo!()` alone can't stand in for an `impl Iterator` return type.
//!
//! Test: cargo test -p api_design --features exercises --bin ex06_iterators_and_generics

#![allow(unused_variables, unreachable_code, dead_code)]

use std::iter::empty;

/// One song.
#[derive(Debug, Clone, PartialEq)]
pub struct Song {
    pub title: String,
    pub seconds: u32,
}

/// Songs in order.
#[derive(Debug, Default)]
pub struct Playlist {
    songs: Vec<Song>,
}

impl Playlist {
    /// An empty playlist.
    pub fn new() -> Self {
        Playlist::default()
    }

    /// Adds a song at the end.
    pub fn add(&mut self, title: impl Into<String>, seconds: u32) -> &mut Self {
        todo!()
    }

    /// The songs, borrowed.
    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        todo!()
    }

    /// The titles.
    pub fn titles(&self) -> impl Iterator<Item = &str> + '_ {
        todo!();
        empty()
    }

    /// Songs longer than `seconds`.
    pub fn longer_than(&self, seconds: u32) -> impl Iterator<Item = &Song> + '_ {
        todo!();
        empty()
    }

    /// The total length in seconds.
    pub fn total_seconds(&self) -> u32 {
        todo!()
    }

    /// The first song whose title contains `words`, in any letter case.
    pub fn find(&self, words: impl AsRef<str>) -> Option<&Song> {
        todo!()
    }
}

impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a Song;
    type IntoIter = std::slice::Iter<'a, Song>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

impl<S: Into<String>> FromIterator<(S, u32)> for Playlist {
    fn from_iter<I: IntoIterator<Item = (S, u32)>>(pairs: I) -> Self {
        todo!()
    }
}

fn main() {
    let mut list = Playlist::new();
    list.add("Clair de Lune", 300)
        .add(String::from("Bolero"), 900);
    for song in &list {
        println!("{} ({}s)", song.title, song.seconds);
    }
    println!("total: {}s", list.total_seconds());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Playlist {
        let mut list = Playlist::new();
        list.add("Clair de Lune", 300)
            .add(String::from("Bolero"), 900)
            .add("Gymnopedie No. 1", 200);
        list
    }

    #[test]
    fn add_chains_and_iter_borrows() {
        let list = sample();
        assert_eq!(list.iter().len(), 3, "slice::Iter knows its length");
        let first = list.iter().next().unwrap();
        assert_eq!(first.title, "Clair de Lune");
        let mut count = 0;
        for song in &list {
            count += song.seconds / 100;
        }
        assert_eq!(count, 14);
    }

    #[test]
    fn titles_and_filters() {
        let list = sample();
        let titles: Vec<&str> = list.titles().collect();
        assert_eq!(titles, ["Clair de Lune", "Bolero", "Gymnopedie No. 1"]);
        let long: Vec<&str> = list.longer_than(250).map(|s| s.title.as_str()).collect();
        assert_eq!(long, ["Clair de Lune", "Bolero"]);
        assert_eq!(list.total_seconds(), 1400);
    }

    #[test]
    fn find_takes_any_text() {
        let list = sample();
        assert_eq!(list.find("BOLERO").unwrap().seconds, 900);
        assert_eq!(list.find(String::from("de lune")).unwrap().seconds, 300);
        assert!(list.find("Requiem").is_none());
    }

    #[test]
    fn collect_into_a_playlist() {
        let list: Playlist = [("a", 1), ("b", 2)].into_iter().collect();
        assert_eq!(list.total_seconds(), 3);
        let owned: Playlist = vec![(String::from("c"), 3)].into_iter().collect();
        assert_eq!(owned.titles().collect::<Vec<_>>(), ["c"]);
    }
}
