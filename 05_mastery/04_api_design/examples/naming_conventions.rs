//! Naming: `as_` / `to_` / `into_`, getters without `get_`, `iter` /
//! `iter_mut` / `into_iter`, and constructor names.
//!
//! Run: cargo run -p api_design --example naming_conventions

/// A word that is never empty.
#[derive(Debug, Clone, PartialEq)]
struct Word(String);

impl Word {
    /// Constructors: `new` for the main one; `try_` when it can fail.
    fn try_new(text: &str) -> Option<Word> {
        (!text.is_empty()).then(|| Word(text.to_string()))
    }

    /// `as_`: a free, borrowed VIEW of the same data. No copy, no allocation.
    fn as_str(&self) -> &str {
        &self.0
    }

    /// `to_`: an expensive conversion that makes NEW data. `self` is borrowed,
    /// so the original is still there afterwards.
    fn to_shouting(&self) -> Word {
        Word(self.0.to_uppercase())
    }

    /// `into_`: consumes `self` and hands over what's inside. Usually free.
    fn into_string(self) -> String {
        self.0
    }

    /// A getter is named after what it returns: `len`, not `get_len`.
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// A shelf of words, with the three standard iterator methods.
#[derive(Debug, Default)]
struct Shelf {
    words: Vec<Word>,
}

impl Shelf {
    /// `with_capacity`, `from_…`, `new`: constructors say what they take.
    fn with_capacity(n: usize) -> Self {
        Shelf {
            words: Vec::with_capacity(n),
        }
    }

    fn push(&mut self, word: Word) {
        self.words.push(word);
    }

    /// `iter`: borrows each item, `&T`.
    fn iter(&self) -> std::slice::Iter<'_, Word> {
        self.words.iter()
    }

    /// `iter_mut`: borrows each item mutably, `&mut T`.
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Word> {
        self.words.iter_mut()
    }

    /// `_mut` suffix for the mutable twin of a getter.
    fn first_mut(&mut self) -> Option<&mut Word> {
        self.words.first_mut()
    }
}

/// `into_iter`: gives away each item, `T`. Implementing `IntoIterator` also
/// makes `for word in shelf` work.
impl IntoIterator for Shelf {
    type Item = Word;
    type IntoIter = std::vec::IntoIter<Word>;

    fn into_iter(self) -> Self::IntoIter {
        self.words.into_iter()
    }
}

fn main() {
    // The standard library follows the same rules:
    let text = String::from("hello");
    let bytes: &[u8] = text.as_bytes(); // as_: a view, free
    let upper: String = text.to_uppercase(); // to_: new data, allocates
    println!("as_bytes = {bytes:?}, to_uppercase = {upper}");
    let owned_bytes: Vec<u8> = text.into_bytes(); // into_: `text` is gone, nothing copied
    println!("into_bytes = {owned_bytes:?}");

    let word = Word::try_new("rust").expect("not empty");
    println!(
        "as_str = {}, to_shouting = {:?}, len = {}",
        word.as_str(),
        word.to_shouting(),
        word.len()
    );
    println!("try_new(\"\") = {:?}", Word::try_new(""));

    let mut shelf = Shelf::with_capacity(3);
    for text in ["apple", "banana", "cherry"] {
        shelf.push(Word::try_new(text).unwrap());
    }
    let lengths: Vec<usize> = shelf.iter().map(Word::len).collect();
    println!("iter      -> lengths {lengths:?}");
    for word in shelf.iter_mut() {
        word.0.push('!');
    }
    if let Some(first) = shelf.first_mut() {
        first.0.insert(0, '*');
    }
    let strings: Vec<String> = shelf.into_iter().map(Word::into_string).collect();
    println!("iter_mut, then into_iter -> {strings:?}");
    // `shelf` is gone now: into_iter consumed it.
    println!("{}", word.into_string()); // and so is `word` after this line
}
