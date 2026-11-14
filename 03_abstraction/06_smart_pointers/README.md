<!-- hand-written lesson: gen_course must not overwrite -->
# Smart Pointers

**The Rust Book, chapter 15:** [Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
· Rust by Example: [Box](https://doc.rust-lang.org/rust-by-example/std/box.html), [Rc](https://doc.rust-lang.org/rust-by-example/std/rc.html)
· std: [`std::boxed`](https://doc.rust-lang.org/std/boxed/index.html), [`std::rc`](https://doc.rust-lang.org/std/rc/index.html), [`std::cell`](https://doc.rust-lang.org/std/cell/index.html), [`LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)

A **pointer** is a value that holds the address of some other data. The references you know (`&T`) are the simplest kind: they borrow, and that's all. A **smart pointer** is a struct that acts like a pointer but also **owns** the data and adds an ability: heap storage, several owners, changes through a shared reference, or a cleanup step. You've used two already: `String` and `Vec<T>` are smart pointers. What makes them pointer-like is two traits, `Deref` (so `*` and method calls reach the data) and `Drop` (so cleanup happens by itself).

> 💡 **The big picture: tickets for things kept somewhere else.** A plain reference is **pointing at a toy on someone else's shelf**: you may look, but it isn't yours. A smart pointer is a **ticket you own**. A cloakroom ticket for one coat (`Box`). A family TV that stays on while anyone is still watching (`Rc`). A cookie jar whose lid checks the rules each time it's opened (`RefCell`). And a friend's phone number (`Weak`): you can ring to see if they're still around, but it doesn't make them stay.

| Pointer | Owners | Can change the data? | Rules checked |
|---|---|---|---|
| `&T` / `&mut T` | none (borrows) | `&mut` only | compile time |
| `Box<T>` | one | if the box is `mut` | compile time |
| `Rc<T>` | many | no, read-only | compile time |
| `RefCell<T>` | one | yes, even through `&` | **run time** (can panic) |
| `Rc<RefCell<T>>` | many | yes | run time |
| `Weak<T>` | none (doesn't keep it alive) | via `upgrade()` | run time (`Option`) |

## 1. `Box<T>`: data on the heap

```rust
let b = Box::new(5);   // the 5 lives on the heap; `b` (a pointer) on the stack
println!("b = {b}");   // prints 5: a Box prints like what it holds
                       // when `b` goes out of scope, both are freed
```

Use a `Box` when:
1. A type's size **can't be known** at compile time, but you need a fixed size (recursive types, below).
2. You have a **large** value and want to move it without copying it: moving a `Box` copies 8 bytes, not the data.
3. You want to own a value of **"some type that implements a trait"**: `Box<dyn Trait>` (lesson 03/07).

**Recursive types.** The Book's cons list holds a value and *the rest of the list*. Written directly, a `List` contains a `List`, which contains a `List`… forever. Exercise 1 makes the same mistake with an expression tree:

```text
error[E0072]: recursive type `Expr` has infinite size
  --> 03_abstraction\06_smart_pointers\exercises\ex01_fix_recursive_type.rs:14:1
   |
14 | enum Expr {
   | ^^^^^^^^^
15 |     Num(i64),
16 |     Add(Expr, Expr),
   |         ---- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
16 |     Add(Box<Expr>, Expr),
   |         ++++    +
```

A `Box` is always one pointer wide, whatever it points to, so `Cons(i32, Box<List>)` has a fixed size:

```text
size of i32:            4 bytes
size of Box<i32>:       8 bytes
size of Box<[u8; 1000]>: 8 bytes
size of List:           16 bytes
```

> 💡 **Analogy: the cloakroom ticket.** Your big coat goes into the cloakroom (the heap), and you keep a **small ticket** in your pocket (the `Box`). Every ticket is the same size, however big the coat. A cons list is a coat with a ticket in its pocket for the next coat, and so on: you can't fit a coat inside a coat inside a coat forever, but a pocket can always hold a ticket. When you throw the ticket away, the cloakroom gives the coat away too.

▶ `cargo run -p smart_pointers --example box_cons_list`

## 2. `Deref`: treating smart pointers like references

`*` follows a reference to the value it points at. To make `*` work on your own type, implement `Deref`. The Book builds `MyBox<T>` to show how:

```rust
struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

let y = MyBox::new(5);
assert_eq!(5, *y);   // Rust turns this into *(y.deref())
```

Without the `impl` (a real error from a scratch file):

```text
error[E0614]: type `MyBox<{integer}>` cannot be dereferenced
 --> e0614.rs:5:19
  |
5 |     assert_eq!(5, *y);
  |                   ^^ can't be dereferenced
```

**Deref coercion.** When you pass a `&MyBox<String>` to a function that wants a `&str`, Rust calls `deref` as many times as needed: `&MyBox<String>` → `&String` → `&str`. It's decided at compile time, so it costs nothing:

```rust
fn hello(name: &str) { println!("Hello, {name}!"); }
let m = MyBox::new(String::from("Rust"));
hello(&m);            // instead of hello(&(*m)[..])
```

The rules: `&T` becomes `&U` when `T: Deref<Target = U>`; `&mut T` becomes `&mut U` with `DerefMut`; and `&mut T` may become `&U`. Never the other way: a shared `&` can't turn into a `&mut`.

> 💡 **Analogy: the ticket that works like the coat.** You hand the ticket to the attendant and say "the coat, please" (`*`). The attendant fetches it. **Deref coercion** is a helpful attendant: if you hand over a ticket for a bag that holds a ticket for the coat, they fetch through both without being asked.

▶ `cargo run -p smart_pointers --example deref_my_box`

## 3. `Drop`: running code on cleanup

```rust
impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}
```

`drop` runs by itself when the value goes out of scope. Values are dropped in **reverse** order of creation:

```text
CustomSmartPointers created.
Dropping CustomSmartPointer with data `other stuff`!
Dropping CustomSmartPointer with data `my stuff`!
```

You may **not** call the method yourself, because Rust would call it again at the end of the scope (a *double free*):

```text
error[E0040]: explicit use of destructor method
  --> e0040.rs:15:7
   |
15 |     c.drop();
   |       ^^^^ explicit destructor calls not allowed
   |
help: consider using `drop` function
   |
15 -     c.drop();
15 +     drop(c);
```

To end a value early, call **`std::mem::drop(c)`** (it's in the prelude, so just `drop(c)`). It takes ownership, so `c` can't be used, or dropped, again. Box, `Vec`, `String`, files and locks all use `Drop` to free memory, close files and release locks: you never have to remember to.

> 💡 **Analogy: tidying up as you leave.** Every toy knows how to put itself away when playtime ends (`Drop`), and the last toy out is the first one put away. You can't say "put yourself away" and then keep playing with it. Instead you hand it to the grown-up (`drop(toy)`), and it's gone.

▶ `cargo run -p smart_pointers --example drop_custom_pointer`

## 4. `Rc<T>`: several owners

Sometimes one value really has **several owners**: two lists that share a tail, two classes that share a teacher. A `Box` has exactly one owner. Exercise 2 tries anyway:

```text
error[E0382]: use of moved value: `teacher`
  --> 03_abstraction\06_smart_pointers\exercises\ex02_fix_shared_owner.rs:33:9
   |
24 |     let teacher = Box::new(Teacher {
   |         ------- move occurs because `teacher` has type `Box<Teacher>`, which does not implement the `Copy` trait
...
29 |         teacher,
   |         ------- value moved here
...
33 |         teacher,
   |         ^^^^^^^ value used here after move
```

`Rc<T>` (*reference counted*) keeps a **count of owners**. `Rc::clone(&a)` adds one to the count and returns another pointer to the *same* data. It never copies the data, which is why the Book writes `Rc::clone(&a)` rather than `a.clone()`. When an owner goes away, the count drops by one, and at zero the data is freed:

```text
count after creating a = 1
count after creating b = 2
count after creating c = 3
count after c goes out of scope = 2
```

- `Rc` gives **read-only** access. Shared data that anyone could change would break the borrowing rules: `shared.push(3)` is `error[E0596]: cannot borrow data in an `Rc` as mutable`. Section 5 fixes that.
- `Rc` is for **one thread**. Sending one to another thread is `E0277: Rc<i32> cannot be sent between threads safely`. The thread-safe version is `Arc` (chapter 16).

> 💡 **Analogy: the family TV.** The first person to come in switches the TV on. Everyone who joins adds one to the count of watchers (`Rc::clone`), and everyone who leaves takes one away. The TV only goes off when the **last** watcher leaves. Nobody gets their own copy of the TV, and nobody may take it apart while others are watching (read-only).

▶ `cargo run -p smart_pointers --example rc_shared_list`

## 5. `RefCell<T>` and interior mutability

The borrowing rules say: **either** many `&` **or** one `&mut`, never both. Usually the compiler checks this. **Interior mutability** lets you change data through a `&` reference by checking the same rules **at run time** instead:

| Method | Gives | If the rules are broken |
|---|---|---|
| `cell.borrow()` | `Ref<T>`, acts like `&T` | panics |
| `cell.borrow_mut()` | `RefMut<T>`, acts like `&mut T` | panics |
| `cell.try_borrow_mut()` | `Result`, never panics | returns `Err` |

**Why you'd want it: the Book's mock object.** A `LimitTracker` sends warnings through a `Messenger` trait, whose `send` takes `&self`. A test wants a mock messenger that *records* what it was sent, which means changing itself through `&self`. Exercise 3 has the same shape with a `Logger`:

```text
error[E0596]: cannot borrow `self.lines` as mutable, as it is behind a `&` reference
  --> 03_abstraction\06_smart_pointers\exercises\ex03_fix_interior_mutability.rs:24:9
   |
24 |         self.lines.push(message.to_string());
   |         ^^^^^^^^^^ `self` is a `&` reference, so it cannot be borrowed as mutable
   |
help: consider changing this to be a mutable reference in the `impl` method and the `trait` definition
   |
15 ~     fn log(&mut self, message: &str);
```

The compiler suggests changing the trait, but the trait isn't yours to change. The fix is `lines: RefCell<Vec<String>>` and `self.lines.borrow_mut().push(…)`.

**Breaking the rules is a panic, not a compile error.** Two `borrow_mut()`s at once compile fine and fail when they run:

```text
$ cargo run -p smart_pointers --example refcell_mock_messenger -- panic
thread 'main' (23588) panicked at 03_abstraction\06_smart_pointers\examples\refcell_mock_messenger.rs:102:35:
RefCell already borrowed
```

The usual way to hit this by accident is to keep a `borrow()` alive while calling something that wants `borrow_mut()`. For example, `match dir.borrow().get(name) { None => dir.borrow_mut()… }` panics, because the `borrow()` lives until the end of the whole `match`. The mini project's `file_at` takes the borrow in a `let` statement of its own, so it ends straight away.

> 💡 **Analogy: the cookie jar with a clever lid.** The rule is "lots of people may **look** in the jar, or **one** person may **reach** in, never both". Normally a grown-up checks that before anyone goes near the kitchen (the compiler). `RefCell` puts the check **in the lid**: it counts who's looking or reaching each time it opens. Break the rule, and the lid snaps shut and sounds the alarm (a panic).

▶ `cargo run -p smart_pointers --example refcell_mock_messenger` and `cargo test -p smart_pointers --example refcell_mock_messenger`

### `Rc<RefCell<T>>`: several owners who can all change it

Put a `RefCell` inside an `Rc`: the `Rc` gives several owners, the `RefCell` lets any of them change the value. Change it through one owner, and everyone sees it:

```text
a after = Cons(RefCell { value: 15 }, Nil)
b after = Cons(RefCell { value: 3 }, Cons(RefCell { value: 15 }, Nil))
c after = Cons(RefCell { value: 4 }, Cons(RefCell { value: 15 }, Nil))
```

`Cell<T>` and `Mutex<T>` (chapter 16) are other kinds of interior mutability. For threads, the pair is `Arc<Mutex<T>>`.

> 💡 **Analogy: the family whiteboard.** Everyone in the house can see it (`Rc`), and anyone can write on it, but there's **one marker** (`RefCell`): you take the marker, write, and put it back. Whatever you write, everybody sees.

▶ `cargo run -p smart_pointers --example rc_refcell_shared`

## 6. Reference cycles can leak memory

`Rc` frees data when the count reaches **zero**. If `a` owns `b` and `b` owns `a`, neither count ever gets there:

```text
== a cycle: a -> b -> a
a count = 2, b count = 2
a and b dropped: each count fell to 1, never 0. Leaked.
```

That's a **memory leak**. Rust's safety rules don't forbid leaks (`Box::leak` and `std::mem::forget` are safe functions); they forbid using freed memory. The example also gives two values in a cycle a noisy `Drop`, and it never prints. Printing a cycle with `{:?}` follows it round and round until the stack overflows.

**The fix is `Weak<T>`**, a pointer that doesn't own:

- `Rc::downgrade(&rc)` makes a `Weak` and adds one to the **weak count**, which doesn't keep the data alive.
- `weak.upgrade()` returns `Option<Rc<T>>`: `Some` if the data is still there, `None` if it has been freed.
- Rule of thumb: **owners point down, `Weak` points back up.** In a tree, a parent owns its children (`Rc`), and each child knows its parent through a `Weak`.

```text
leaf parent = Some(5)
branch has 1 child
branch strong = 1, weak = 1
leaf strong = 2, weak = 0
after branch's scope: leaf parent = None
```

The branch is freed even though the leaf still points at it, because that pointer is weak. Exercise 6 builds a family tree like this and checks that nothing leaks.

> 💡 **Analogy: two friends holding each other's balloons.** Amy says "I'll let go of Ben's balloon when Ben lets go of mine", and Ben says the same. Nobody ever lets go, and the balloons are stuck there for ever (a leak). With `Weak`, Ben doesn't hold Amy's balloon; he just has **Amy's phone number**. He can ring to check she's still there (`upgrade`), but it doesn't stop her going home.

▶ `cargo run -p smart_pointers --example reference_cycles`

## Beyond the Book: `Cell<T>` and `OnceCell<T>`

| Type | Use it for | Main methods |
|---|---|---|
| `Cell<T>` | small `Copy` values changed through `&self`: counters, flags | `get`, `set`, `replace`, `take` |
| `RefCell<T>` | anything, when you need a reference to the inside | `borrow`, `borrow_mut` |
| `OnceCell<T>` | a value set once, read many times: lazy caches | `get_or_init`, `set`, `get` |

- A `Cell` **never hands out a reference** to what's inside: you copy the value out (`get`) or swap a new one in (`set`). So there's nothing to track and nothing that can panic, and it costs nothing.
- An `OnceCell` starts empty. `get_or_init(|| …)` runs the closure the first time and returns the stored value every time after. A second `set` returns `Err`.

```text
Smart pointers was viewed 3 times
replace: old 10, now 20
  (counting words, once)
words: 6
words: 6
set again:  Err("light mode")
```

> 💡 **Analogy:** a `Cell` is a **scoreboard with number cards**: you never write on a card, you just swap the whole card for a new one. A `OnceCell` is a **name tag written in permanent marker**: blank until someone writes the name, and then it never changes.

▶ `cargo run -p smart_pointers --example cell_and_oncecell`

## Beyond the Book: `LazyLock` and `OnceLock` for lazy globals

A `static` has to be built at compile time, so it can't hold a `HashMap`, a `String` read from a file, or anything else made at run time. These two build the value **the first time it's used, exactly once**, even if several threads ask at the same moment:

```rust
static CAPITALS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([("Ghana", "Accra"), ("Kenya", "Nairobi"), ("Peru", "Lima")])
});

static CONFIG: OnceLock<String> = OnceLock::new();   // set later: CONFIG.set(…) or get_or_init
```

```text
main starts; CAPITALS hasn't been built yet
  (building CAPITALS: this happens once)
capital of Ghana: Accra
capital of Peru:  Lima

set CONFIG: Ok(())
set again:  Err("too late")
```

- `LazyLock` has its recipe **next to the static**. `OnceLock` is filled **by the program** once it knows the value (from arguments or a config file, say).
- One-thread versions: `std::cell::LazyCell` and `OnceCell`. Older code uses the `lazy_static` or `once_cell` crates for the same thing.

> 💡 **Analogy:** `LazyLock` is the school's **lost-property box**: nobody builds it until the first time someone loses something, and then there's one box for the whole school. `OnceLock` is the **class photo**: taken once, on the day everyone's there, and never retaken.

▶ `cargo run -p smart_pointers --example lazy_globals`

## The mini project: `tinyfs`

`src/` is a pretend file system that lives in memory, with a tiny shell. Every pointer in the chapter has a real job:

| Pointer | Its job in tinyfs |
|---|---|
| `Box<T>` | the command `History` is the Book's cons list (`src/history.rs`) |
| `Deref` | a checked `Name` works anywhere a `&str` does (`src/fs.rs`) |
| `Drop` | a freed `Dir` writes `freed directory …` to a log |
| `Rc<T>` | `ln` lists one `File` in two directories: a hard link, and `stat` shows the count |
| `RefCell<T>` | directories and files change while they're shared |
| `Rc<RefCell<T>>` | the drop log, shared by every directory |
| `Weak<T>` | each directory's parent link, so `cd ..` works and nothing leaks |
| `Cell<T>`, `OnceCell<T>` | a file's read counter; a directory's cached path |
| `LazyLock<T>` | the global table of commands for `help` |

```bash
cargo run -p smart_pointers           # a scripted demo session
cargo run -p smart_pointers -- -      # type your own commands
cargo test -p smart_pointers          # 14 unit tests + 1 doc test
```

```text
tinyfs:/projects/rust$ ln notes.txt /
notes.txt: 2 links
tinyfs:/$ tree
/
├── notes.txt (2 links)
└── projects/
    └── rust/
        └── notes.txt (2 links)
tinyfs:/$ stat projects
directory /projects: entries 1, Rc owners 1, Weak refs 1
tinyfs:/$ rm projects
removed projects/
freed directory projects
freed directory rust
tinyfs:/$ stat notes.txt
file: 65 bytes, 1 link(s), read 1 time(s)
tinyfs:/$ cat notes.txt
Box owns one value on the heap.
Rc counts owners. Weak does not.

(the shell is dropped)
freed directory /
```

`rm projects` frees both directories at once: `Drop` runs on the parent first, then on its fields, which include the subdirectory. The file inside survives, because the root still owns it through the second link. When the shell is dropped, the root goes too, and the tests check that **every** directory gets freed. That only works because parent links are `Weak`.

> 💡 **Analogy: a tree house with name tags.** Each room owns the rooms built off it, and each room has a **note with the address of the room below** (`Weak`), not a rope tying it down. Knock out a room, and everything built off it comes down too, with a "freed!" shout from each one. A toy box with two name tags (a hard link) stays as long as **any** tag is still on a door.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `enum List { Cons(i32, List), Nil }` | E0072 infinite size | `Box<List>` (or `Rc<List>` to share) |
| `*my_type` with no `Deref` impl | E0614 cannot be dereferenced | `impl Deref for MyType` |
| `value.drop()` | E0040 explicit destructor call | `drop(value)` |
| The same `Box` (or value) in two owners | E0382 use of moved value | `Rc::new` once, then `Rc::clone(&rc)` for each owner |
| Changing data inside an `Rc` | E0596 cannot borrow data in an `Rc` as mutable | `Rc<RefCell<T>>` |
| Pushing to a field in a `&self` method | E0596 behind a `&` reference | Wrap the field in `RefCell` (or `Cell` for `Copy` values) |
| Two `borrow_mut()`s alive at once | panic: RefCell already borrowed | Keep borrows short: their own statement or a `{}` block. Or `try_borrow_mut`. |
| `use std::borrow::Borrow;` next to `RefCell` | E0282 type annotations needed on `.borrow()` | Don't import the trait; write `std::borrow::Borrow` in the `impl`. |
| `a.clone()` on an `Rc` | works, but looks like a deep copy | `Rc::clone(&a)` says what it does |
| Parent and child both hold `Rc`s to each other | silent memory leak | Child → parent with `Weak` |
| `weak.upgrade().unwrap()` | panics once the data is gone | `if let Some(rc) = weak.upgrade()` |
| `Rc` or `RefCell` sent to another thread | E0277 cannot be sent between threads safely | `Arc` and `Mutex` (chapter 16) |
| `{:?}` on a structure with a cycle | stack overflow | Break the cycle with `Weak`, or write your own `Debug`. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p smart_pointers --example box_cons_list
cargo run -p smart_pointers --example deref_my_box
cargo run -p smart_pointers --example drop_custom_pointer
cargo run -p smart_pointers --example rc_shared_list
cargo run -p smart_pointers --example refcell_mock_messenger
cargo test -p smart_pointers --example refcell_mock_messenger
cargo run -p smart_pointers --example refcell_mock_messenger -- panic
cargo run -p smart_pointers --example rc_refcell_shared
cargo run -p smart_pointers --example reference_cycles
cargo run -p smart_pointers --example cell_and_oncecell
cargo run -p smart_pointers --example lazy_globals
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_recursive_type` | Make it compile (E0072) with `Box` | `cargo run -p smart_pointers --features exercises --bin ex01_fix_recursive_type` |
| 2 | `ex02_fix_shared_owner` | Make it compile (E0382) with `Rc` | `cargo run -p smart_pointers --features exercises --bin ex02_fix_shared_owner` |
| 3 | `ex03_fix_interior_mutability` | Make it compile (E0596) with `RefCell`, trait unchanged | `cargo run -p smart_pointers --features exercises --bin ex03_fix_interior_mutability` |
| 4 | `ex04_boxed_stack` | A stack of `Option<Box<Node>>`, pass 3 tests | `cargo test -p smart_pointers --features exercises --bin ex04_boxed_stack` |
| 5 | `ex05_deref_and_drop` | Your own smart pointer: `Deref`, `DerefMut`, `Drop`, pass 4 tests | `cargo test -p smart_pointers --features exercises --bin ex05_deref_and_drop` |
| 6 | `ex06_weak_tree` | A family tree with `Weak` parents and no leaks, pass 3 tests | `cargo test -p smart_pointers --features exercises --bin ex06_weak_tree` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p smart_pointers --features solutions --bins`).

**4. The mini project:** run the demo, then type your own session with `cargo run -p smart_pointers -- -`. Read `src/fs.rs` for the pointers and `src/shell.rs` for how they're used. Then try:
- Make the parent link an `Rc` instead of a `Weak` (a few lines in `src/fs.rs`). Which tests fail, and why?
- Add a `cp` command that **copies** a file (a new `Rc<File>` with the same text), and compare `stat` after `cp` and after `ln`.

## Checklist

- [ ] [15.1 Using Box<T> to Point to Data on the Heap](https://doc.rust-lang.org/book/ch15-01-box.html)
- [ ] [15.2 Treating Smart Pointers Like Regular References](https://doc.rust-lang.org/book/ch15-02-deref.html)
- [ ] [15.3 Running Code on Cleanup with the Drop Trait](https://doc.rust-lang.org/book/ch15-03-drop.html)
- [ ] [15.4 Rc<T>, the Reference Counted Smart Pointer](https://doc.rust-lang.org/book/ch15-04-rc.html)
- [ ] [15.5 RefCell<T> and the Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
- [ ] [15.6 Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
- [ ] Beyond the Book: `Cell<T>`, `OnceCell<T>`
- [ ] Beyond the Book: `LazyLock`/`OnceLock` for lazy globals
- [ ] Exercises 1–6 and the tinyfs mini project done

## Recap

- A smart pointer owns its data and adds an ability. `Deref` makes it act like a reference, and `Drop` cleans up by itself.
- `Box<T>`: one owner, data on the heap, a fixed size. It's what makes recursive types possible.
- `Rc<T>`: many owners, read-only, freed when the count reaches zero. One thread only.
- `RefCell<T>`: change data through `&` by checking the borrowing rules at run time. `Rc<RefCell<T>>` gives many owners who can all change it.
- Cycles of `Rc` leak. Point back up with `Weak<T>`, and `upgrade()` when you need it.
- `Cell` swaps small values, `OnceCell` is set once, and `LazyLock`/`OnceLock` build globals on first use.

**In one sentence for a child:** a smart pointer is a ticket you own for something kept somewhere else: one ticket per coat (`Box`), a TV that stays on while anyone's watching (`Rc`), a cookie jar whose lid checks the rules (`RefCell`), and a friend's phone number that doesn't make them stay (`Weak`).

**Next:** [07 Trait Objects and OOP](../07_trait_objects_and_oop/README.md) (Book chapter 18; chapters 16 and 17, on threads and async, come in [04 Systems Level](../../04_systems_level/README.md))
