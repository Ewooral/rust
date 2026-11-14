//! Beyond the Book: `Cell<T>` and `OnceCell<T>`, RefCell's simpler cousins.
//!
//! Run: cargo run -p smart_pointers --example cell_and_oncecell

use std::cell::{Cell, OnceCell};

/// A page that counts its visitors through `&self`.
struct Page {
    title: String,
    views: Cell<u32>,
    /// Worked out the first time someone asks, then kept for ever.
    word_count: OnceCell<usize>,
    body: String,
}

impl Page {
    fn view(&self) -> &str {
        // Cell: copy the number out, add one, put it back. No borrows, so
        // nothing to check at run time and nothing that can panic.
        self.views.set(self.views.get() + 1);
        &self.title
    }

    fn word_count(&self) -> usize {
        *self.word_count.get_or_init(|| {
            println!("  (counting words, once)");
            self.body.split_whitespace().count()
        })
    }
}

fn main() {
    let page = Page {
        title: String::from("Smart pointers"),
        views: Cell::new(0),
        word_count: OnceCell::new(),
        body: String::from("Box Rc RefCell Weak Cell OnceCell"),
    };

    // 1. Cell<T>: get() copies the value out, set() puts a new one in.
    //    Only for Copy types with get(); you never get a reference inside.
    for _ in 0..3 {
        page.view();
    }
    println!("{} was viewed {} times", page.title, page.views.get());

    let cell = Cell::new(10);
    let old = cell.replace(20); // swap in a new value, get the old one
    println!("replace: old {old}, now {}", cell.get());
    let taken = cell.take(); // take the value, leave Default (0) behind
    println!("take: got {taken}, left {}", cell.get());

    // 2. OnceCell<T>: empty until first set, then never changes.
    println!("words: {}", page.word_count());
    println!("words: {}", page.word_count()); // no second "(counting...)"

    let setting: OnceCell<&str> = OnceCell::new();
    println!("set first:  {:?}", setting.set("dark mode"));
    println!("set again:  {:?}", setting.set("light mode")); // Err: already set
    println!("value:      {:?}", setting.get());
}

// Which cell?
//   Cell<T>      small Copy values (counters, flags): get / set / replace
//   RefCell<T>   anything; borrow() / borrow_mut() hand out references,
//                checked at run time (can panic)
//   OnceCell<T>  set once, read many times: lazy caches and late set-up
// All three are for ONE thread. Thread-safe versions: atomics, Mutex,
// RwLock, OnceLock (see the lazy_globals example).
