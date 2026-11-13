<!-- hand-written lesson: gen_course must not overwrite -->
# Collections

**The Rust Book, chapter 8:** [Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
· Rust by Example: [Vectors](https://doc.rust-lang.org/rust-by-example/std/vec.html), [Strings](https://doc.rust-lang.org/rust-by-example/std/str.html), [HashMap](https://doc.rust-lang.org/rust-by-example/std/hash.html)
· [`std::collections`](https://doc.rust-lang.org/std/collections/index.html) (with a "when to use which" guide)

Arrays and tuples have a fixed size known at compile time, and they live on the stack. **Collections** store their data on the **heap**, so they can grow and shrink while the program runs. This chapter covers the three you will use every day:

| Collection | Holds | Find things by |
|---|---|---|
| `Vec<T>` | a list of values of one type, in order | position: `v[2]` |
| `String` | UTF-8 text | (not by position! see section 2) |
| `HashMap<K, V>` | pairs of key → value | key: `scores.get("Blue")` |

> 💡 **The big picture: storage at home.** A `Vec` is a **row of numbered lockers**, side by side, and you can add more at the end. A `String` is a **bead necklace**: the beads are characters, and some beads are bigger than others. A `HashMap` is a **coat check**: you don't remember *where* your coat hangs, only your ticket (the key), and the attendant finds it instantly.

## 1. `Vec<T>`: lists of values

```rust
let mut v: Vec<i32> = Vec::new();  // empty: needs the type
let w = vec![1, 2, 3];             // the macro infers Vec<i32>
v.push(5);                         // the Vec must be `mut`
let third: &i32 = &v[2];           // panics if out of range
let maybe: Option<&i32> = v.get(2); // None if out of range
```

- **`[i]` or `.get(i)`?** Use `[i]` when an out-of-range index is a bug and the program *should* crash. Use `.get(i)` when it can legitimately happen, like an index typed in by a user, and handle the `None`.
- **Iterate** with `for x in &v` (read) or `for x in &mut v { *x += 1 }` (change). `*` follows the reference to the value.
- A Vec holds **one type**. When you need several kinds, use an **enum** whose variants hold the different kinds: `Vec<SpreadsheetCell>`.
- Dropping a Vec drops everything inside it.

> 💡 **Analogy: a row of lockers.** All the lockers are the same size (one type) and stand side by side, numbered from 0. `push` bolts a new locker on the end. `v[7]` walks straight to locker 7, and if there are only 3, you walk into a wall (a panic). `v.get(7)` asks the caretaker first, who says "no such locker" (`None`).

**The borrowing rules still apply.** While you hold a reference to one element, you cannot push:

```text
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
   |
14 |       for s in &scores {
   |                -------
   |                |
   |                immutable borrow occurs here
   |                immutable borrow later used here
15 | |         if *s > 50 {
16 | |             scores.push(s * 2);
   | |             ^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

(This is the real error from exercise 1.) Why should one element care about the end of the Vec? If there is no room left, `push` **moves every element** to a new, bigger block of memory and frees the old one, so your reference would point at freed memory.

> 💡 **Analogy:** when the row of lockers is full, the school doesn't bolt on one more; it moves *everything* into a longer row in another corridor. If you were holding the key to "locker 0 in the old corridor", your key would open nothing. So Rust won't let anyone build while you hold a key.

▶ `cargo run -p collections --example vectors`

## 2. `String`: UTF-8 text

`String` is a growable, owned, **UTF-8** encoded string. Under the hood it is a `Vec<u8>` of bytes. `&str` is a borrowed view of string data (lesson 01/06).

```rust
let mut s = String::from("foo");  // or "foo".to_string()
s.push_str("bar");                // append a &str (borrowed, not moved)
s.push('!');                      // append one char

let s1 = String::from("Hello, ");
let s2 = String::from("world!");
let s3 = s1 + &s2;                // s1 is MOVED; s2 is borrowed
let s4 = format!("{s2}-{s3}");    // takes nothing, returns a new String
```

`+` uses `fn add(self, s: &str) -> String`. It takes ownership of the left side, appends a copy of the right side, and hands the same buffer back, so no new allocation is needed when there is room. `&String` turns into `&str` automatically (deref coercion). For anything longer than `a + &b`, use `format!`.

> 💡 **Analogy: gluing pages.** `s1 + &s2` is handing over *your* notebook (it's no longer yours), having a copy of your friend's page glued in at the end, and getting the notebook back. Your friend keeps their page. `format!` is a photocopier: it makes a brand-new page from everyone's text and leaves all the originals alone.

### Why you can't write `s[0]`

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
   |
11 |         let first = name[0];
   |                          ^ string indices are ranges of `usize`
   = note: you can use `.chars().nth()` or `.bytes().nth()`
```

(This is the real error from exercise 2.) In UTF-8 one character takes **1 to 4 bytes**: `"Hola"` is 4 bytes, but `"Здравствуйте"` is 24 bytes for 12 letters. "The byte at index 0" might be half a letter. Rust refuses to guess what you meant, and it won't hide a slow character-by-character search behind something that looks like fast indexing.

There are three ways to look at text, so pick the one you mean:

| Want | Use | `"नमस्ते"` gives |
|---|---|---|
| bytes | `.bytes()`, `.len()` | 18 bytes |
| Unicode scalar values | `.chars()`, `.chars().count()` | 6 chars |
| letters as people see them (grapheme clusters) | the `unicode-segmentation` crate | 4 |

Slicing with a **byte range**, as in `&s[0..4]`, works, but it panics at runtime if the range cuts a character in half.

> 💡 **Analogy: a bead necklace with beads of different sizes.** English letters are small beads (1 byte), many other letters are bigger beads (2 to 4 bytes). "Give me bead number 3" is easy to answer by counting beads (`chars`). "Give me the bead at millimetre 3" might land in the middle of a big bead. That's what byte indexing would be, so Rust doesn't offer it.

▶ `cargo run -p collections --example strings`

## 3. `HashMap<K, V>`: keys and values

```rust
use std::collections::HashMap;            // not in the prelude

let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10);
let blue = scores.get("Blue").copied().unwrap_or(0);  // get → Option<&V>
for (team, score) in &scores { /* arbitrary order! */ }
```

**Ownership:** `Copy` values (like `i32`) are copied in. Owned values (like `String`) are **moved** into the map, and the map owns them from then on. The real error from exercise 3:

```text
error[E0382]: borrow of moved value: `name`
   |
14 |     ages.insert(name, 9);
   |                 ---- value moved here
15 |     println!("added {name}, the map now has {} entries", ages.len());
   |                      ^^^^ value borrowed here after move
help: consider cloning the value if the performance cost is acceptable
```

The compiler suggests `.clone()`, and sometimes that's right. Usually, though, the better fix is to use the value *before* you hand it over, or to read it back from the map, which now owns it.

> 💡 **Analogy: a coat check.** You hand your coat (`String`) over the counter, and now the coat check has it, not you. You can always ask for it back by name (`get`), but you can't wear it while it hangs there. Lending a photo of your coat (`&str` keys) instead means the photo must not outlive the coat (lifetimes, lesson 03/01).

### Updating a value

| Goal | Code |
|---|---|
| Overwrite | `scores.insert(key, 25);` |
| Insert only if missing | `scores.entry(key).or_insert(50);` |
| Change based on the old value | `*map.entry(word).or_insert(0) += 1;` |

`entry` returns an `Entry` that says whether the key is there. `or_insert` returns a **`&mut V`** to the value, whether it was there or newly inserted, so you can change it in place. This is the classic word count:

```rust
let mut counts = HashMap::new();
for word in text.split_whitespace() {
    *counts.entry(word).or_insert(0) += 1;
}
```

> 💡 **Analogy: a tally sheet at the door.** For each guest: "Is your name on the sheet? No? Write it down with 0. Either way, add a tick next to it." `entry(...).or_insert(0)` finds or writes the line, and `+= 1` is the tick.

**Hashing:** the default hasher, SipHash 1-3, is not the fastest, but it resists attackers who send keys designed to collide and slow your server down. Other hashers are available as crates if you profile and need speed.

▶ `cargo run -p collections --example hash_maps`

## Beyond the Book: the other collections

| Use | When you need | Analogy |
|---|---|---|
| `VecDeque<T>` | a queue: push and pop at **both** ends cheaply | a bus queue with doors at the front and the back |
| `HashSet<T>` | unique values, fast "have I seen this?" | a guest list where each name appears once |
| `BTreeMap<K, V>` / `BTreeSet<T>` | like the Hash versions, but **kept sorted**, with range queries | a dictionary in alphabetical order |
| `BinaryHeap<T>` | always take the **largest** next (a priority queue) | hospital triage: the most urgent patient goes first |

Rule of thumb: start with `Vec` or `HashMap`. Switch when you need what the others give you: order (`BTree*`), uniqueness (`*Set`), two-ended queues (`VecDeque`) or priority (`BinaryHeap`).

▶ `cargo run -p collections --example more_collections`

## Beyond the Book: the Vec toolkit

`with_capacity`, `extend`, `contains`, `sort` / `sort_by_key` / `sort_by`, `dedup` (sort first!), `retain`, `binary_search` (on sorted data), `windows`, `chunks`, `remove` vs `swap_remove`. The example also shows **two pointers**, a classic technique: on sorted data, one index starts at each end and they walk inward. It finds a pair with a given sum in one pass instead of checking every pair.

> 💡 **Analogy: capacity.** `Vec::with_capacity(10)` books ten lockers in advance, even though only three are in use. Pushing up to ten never has to move house.

▶ `cargo run -p collections --example vec_toolkit`

## Beyond the Book: `Cow`, paths and OS strings

- **`Cow<'_, str>`** (clone on write) holds either a borrowed `&str` or an owned `String`. A function that *usually* returns its input unchanged can borrow in the common case and allocate only when it really changes something.
- **`Path` / `PathBuf`** are to file paths what `&str` / `String` are to text. They handle `\` on Windows and `/` on Linux for you.
- **`OsStr` / `OsString`** hold text exactly as the operating system gives it, which might not be valid UTF-8. Convert with `to_str()`, which returns an `Option`, or `to_string_lossy()`.

> 💡 **Analogy: `Cow` is borrowing a library book and only buying your own copy if you need to write in it.** Most of the time you just read, so you never pay.

▶ `cargo run -p collections --example cow_and_paths`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `push` while holding a reference or iterating | E0502 | Collect changes into a new Vec, then `extend` after the loop. |
| `v[i]` with a bad index | panic | `v.get(i)` and handle `None`. |
| `s[0]` on a String | E0277 | `s.chars().next()` or `s.chars().nth(i)` |
| `&s[0..1]` on non-ASCII text | panic: not a char boundary | Slice on `char_indices()` positions, or work with `chars()`. |
| Used `s1` after `s1 + &s2` | E0382 | `format!("{s1}{s2}")` |
| `s1 + s2` (both `String`) | E0308 | `s1 + &s2` |
| Used a key after `insert(key, …)` | E0382 | Use it first, or read it back from the map. |
| Printed a HashMap and expected a fixed order | none, but the order varies | Collect into a Vec and sort, or use `BTreeMap`. |
| `dedup()` on unsorted data | none, but duplicates remain | `sort()` first: `dedup` removes only **neighbouring** duplicates. |
| Forgot `use std::collections::HashMap;` | E0433 | Add the `use`. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p collections --example vectors
cargo run -p collections --example strings
cargo run -p collections --example hash_maps
cargo run -p collections --example more_collections
cargo run -p collections --example vec_toolkit
cargo run -p collections --example cow_and_paths
```

**2. Do the exercises** in `exercises/`. Numbers 4 and 5 are the exercises the Book itself suggests at the end of chapter 8:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_push_while_iterating` | Make it compile (E0502) | `cargo run -p collections --features exercises --bin ex01_fix_push_while_iterating` |
| 2 | `ex02_fix_string_index` | Make it compile (E0277), UTF-8 safe | `cargo run -p collections --features exercises --bin ex02_fix_string_index` |
| 3 | `ex03_fix_moved_into_map` | Make it compile (E0382), no `.clone()` | `cargo run -p collections --features exercises --bin ex03_fix_moved_into_map` |
| 4 | `ex04_median_and_mode` | Implement, pass 3 tests (Book) | `cargo test -p collections --features exercises --bin ex04_median_and_mode` |
| 5 | `ex05_pig_latin` | Implement, pass 3 tests (Book) | `cargo test -p collections --features exercises --bin ex05_pig_latin` |
| 6 | `ex06_word_frequency` | Implement with `entry`, pass 3 tests | `cargo test -p collections --features exercises --bin ex06_word_frequency` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p collections --features solutions`).

**4. The mini project** is the Book's third suggested exercise: a **company directory**, a `HashMap<String, Vec<String>>` from department to people, with a text interface.

```bash
cargo run -p collections            # scripted demo
cargo run -p collections -- -i      # type commands yourself: Add Sally to Engineering / List Engineering / List all / quit
cargo test -p collections           # unit tests and doctests
```

- `src/directory.rs` has the code. Notice `entry(...).or_default()`, which creates a department's Vec the first time someone joins it. Also notice how `everyone()` sorts the keys, because a HashMap has no order.
- `src/arrays.rs` and `src/strings.rs` are your earlier helpers, now part of the same library. They're used at the end of the demo, and their doctests run with `cargo test`.

> 💡 **Analogy:** the directory is a filing cabinet. Each drawer is labelled with a department (the key), and inside each drawer is a list of names (a Vec). "Add Sally to Engineering" opens the Engineering drawer, adding a new drawer if there isn't one yet, and drops Sally's card in.

## Checklist

- [ ] [8.1 Storing Lists of Values with Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html)
- [ ] [8.2 Storing UTF-8 Encoded Text with Strings](https://doc.rust-lang.org/book/ch08-02-strings.html)
- [ ] [8.3 Storing Keys with Associated Values in Hash Maps](https://doc.rust-lang.org/book/ch08-03-hash-maps.html)
- [ ] Beyond the Book: `VecDeque`, `HashSet`, `BTreeMap`, `BTreeSet`, `BinaryHeap` and when to use each
- [ ] Beyond the Book: `Cow<'_, str>` (clone-on-write)
- [ ] Beyond the Book: OS strings and paths: `OsString`, `Path`, `PathBuf`
- [ ] Beyond the Book: classic algorithms in Rust: sorting, binary search, two pointers
- [ ] Exercises 1–6 and the company directory done

## Recap

- `Vec<T>` is a growable list of one type. Read with `[i]` (panics) or `.get(i)` (`Option`). You can't push while borrowing an element.
- `String` is owned UTF-8. Grow it with `push_str`, `push`, `+` (which moves the left side) or `format!`. There's no indexing; choose `bytes()` or `chars()` deliberately.
- `HashMap<K, V>` maps keys to values. Inserting moves owned values in. `entry().or_insert()` gives a `&mut V` for update-or-insert.
- Reach for `VecDeque`, `HashSet`, `BTreeMap` or `BinaryHeap` when you need their particular skill.

**In one sentence for a child:** a `Vec` is a row of lockers, a `String` is a necklace of different-sized beads, and a `HashMap` is a coat check where you find things by name.

**Next:** [05 Error Handling](../05_error_handling/README.md) (Book chapter 9)
