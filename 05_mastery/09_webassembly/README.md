<!-- hand-written lesson: gen_course must not overwrite -->
# WebAssembly

**Books:** [The Rust and WebAssembly book](https://rustwasm.github.io/docs/book/) · [The `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/) · [`wasm-pack` docs](https://rustwasm.github.io/docs/wasm-pack/)
· Specs and runtimes: [webassembly.org](https://webassembly.org/), [MDN: WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly), [WASI](https://wasi.dev/), [wasmtime](https://docs.wasmtime.dev/)
· The rustc book: [`wasm32-unknown-unknown`](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html), [`wasm32-wasip1`](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html), [`wasm32-wasip2`](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip2.html)
· API docs: [`wasm-bindgen`](https://docs.rs/wasm-bindgen)

Run it: `cargo run -p webassembly`  ·  Test it: `cargo test -p webassembly`

**WebAssembly** (wasm) is a small, fast, sandboxed bytecode that every modern browser runs, and that also runs outside browsers (Node.js, wasmtime, edge servers, plugin systems). Rust is one of the best languages to write it in: no garbage collector to ship, tiny output, and the same code you test on your PC. After this lesson you'll be able to:

- explain what a `.wasm` module is, and compile Rust for `wasm32-unknown-unknown` (browsers) and `wasm32-wasip1`/`wasip2` (WASI);
- call Rust from JavaScript and JavaScript from Rust with `wasm-bindgen`, and pass numbers, strings, arrays and structs between them;
- keep the core logic testable on the PC, with JavaScript behind a trait and `cfg(target_arch = "wasm32")`;
- build a browser page with `wasm-pack`, make the output small, and run a program under WASI.

**What was run here.** `rustup target list --installed` on this machine printed only `x86_64-pc-windows-msvc`, and neither `wasm-pack` nor `wasm-bindgen-cli` is installed. So everything that *needs* the wasm target (the `.wasm` build, the browser page, wasmtime) is shown with exact commands and clearly marked **not run here**. Everything else was run: the Game of Life core and its tests, every example, the exercises, and the `#[wasm_bindgen]` code itself, because the `wasm-bindgen` crate (0.2.130 here) compiles on the PC and **checks its signatures there too**.

> 💡 **The big picture: a toy robot in a glass box.** Your Rust program is a robot that can do maths very fast. WebAssembly is a **glass box** you put it in: it can't touch anything outside, so any browser happily lets it in. The box has a **letterbox** (exports and imports) for passing notes in and out, and one **shelf** inside (linear memory) where both sides can put things. `wasm-bindgen` is the **helper at the letterbox** who knows how to fold a long letter (a string) so it fits through, and unfold the answer on the other side.

## 1. What a `.wasm` file is

A WebAssembly **module** is a list of sections: types, imported and exported functions, memory, code. The code is bytecode for a simple **stack machine**: instructions push and pop values. The example holds a complete 41-byte module, written by hand, that exports `add`, then decodes it and runs it with a tiny interpreter:

```text
magic [00, 61, 73, 6D] ("\0asm"), version 1
section  1 (type    ) 7 bytes: [01, 60, 02, 7F, 7F, 01, 7F]
section  3 (function) 2 bytes: [01, 00]
section  7 (export  ) 7 bytes: [01, 03, 61, 64, 64, 00, 00]
    exports "add"
section 10 (code    ) 9 bytes: [01, 07, 00, 20, 00, 20, 01, 6A, 0B]

Running add(2, 3) on a stack machine:
    local.get 0   stack: [2]
    local.get 1   stack: [2, 3]
    i32.add       stack: [5]
    result: 5
```

- Only **four value types** cross a function call: `i32`, `i64`, `f32`, `f64`. Everything else (strings, structs) has to be built on top of numbers.
- A module has **linear memory**: one resizable array of bytes. Rust's stack, heap and statics all live in it, and JavaScript can read and write it too.
- It's a **sandbox**: a module can't touch the screen, files or network by itself, only call the functions it was given (imports).

> 💡 **Analogy: a calculator with a notepad.** The robot has a stack of number cards (it can only add the top two, and so on) and one notepad (linear memory). Everything it knows how to do is in a tiny instruction booklet: 41 bytes for "add two numbers".

▶ `cargo run -p webassembly --example wasm_module_bytes`

## 2. Compiling Rust to wasm32

| Target | Runs in | `std` |
|---|---|---|
| `wasm32-unknown-unknown` | browsers, Node.js (with `wasm-bindgen`) | compiles, but there's no OS: `SystemTime::now()` and `Instant::now()` panic with "time not implemented on this platform" (from std's source, `sys/time/unsupported.rs`), and there are no files or threads |
| `wasm32-wasip1` (formerly `wasm32-wasi`) | wasmtime, Wasmer, Node's WASI | works: files (in allowed folders), args, env, clock, stdout |
| `wasm32-wasip2` | wasmtime, component-model hosts | like p1, built on the newer component model |
| `wasm32v1-none` | anywhere | `no_std` only (lesson 08) |

This toolchain's `rustup target list` also lists `wasm32-wasip1-threads`, `wasm32-wasip3` and `wasm32-unknown-emscripten`. A browser library also needs a **`cdylib`** crate type, the kind of library a `.wasm` module is; `rlib` keeps it usable from normal Rust (this lesson's `main.rs`, examples and tests):

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
```

To build the raw module (**not run here**):

```bash
rustup target add wasm32-unknown-unknown
cargo build -p webassembly --lib --target wasm32-unknown-unknown --release
# -> target/wasm32-unknown-unknown/release/webassembly.wasm
```

> 💡 **Analogy: building for a different country.** The same recipe can be cooked in different kitchens. `wasm32-unknown-unknown` is a kitchen with no fridge and no clock (the browser lends you its own, through JavaScript). A WASI kitchen has a small fridge and a clock, but you choose which cupboards it may open.

## 3. `wasm-bindgen`: Rust from JavaScript, and JavaScript from Rust

```rust
use wasm_bindgen::prelude::*;

// JavaScript -> Rust: imports
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(message: &str);
    #[wasm_bindgen(js_namespace = Math, js_name = random)]
    pub fn math_random() -> f64;
}

// Rust -> JavaScript: exports
#[wasm_bindgen(js_name = Universe)]
pub struct WasmUniverse { /* ... */ }

#[wasm_bindgen(js_class = Universe)]
impl WasmUniverse {
    #[wasm_bindgen(constructor)]               // new Universe(64, 48)
    pub fn new(width: u32, height: u32) -> WasmUniverse { /* ... */ }
    #[wasm_bindgen(js_name = fromPattern)]     // Universe.fromPattern(text, w, h), throws on error
    pub fn from_pattern(text: &str, width: u32, height: u32) -> Result<WasmUniverse, JsError> { /* ... */ }
    #[wasm_bindgen(getter)]                    // universe.population (no brackets)
    pub fn population(&self) -> u32 { /* ... */ }
}

#[wasm_bindgen(start)]                         // runs once when the module loads
pub fn start() { /* ... */ }
```

The macro writes the glue on both sides: Rust functions with number-only signatures, and a JavaScript module (`pkg/webassembly.js`) with a real `Universe` class that converts strings and objects for you.

**On the PC**, the exports are ordinary Rust, so tests can call them. The imports have no JavaScript behind them; calling one panics (real output):

```text
cannot call wasm-bindgen imported functions on non-wasm targets
```

`JsError::new` (used for the `Err` side of `Result<_, JsError>`) needs JavaScript too, so it panics on the PC as well. That's why the error *logic* lives in plain Rust (`pattern::parse`), and the binding only converts it.

```text
fromPattern("OOO", 5, 3):
.....
.###.
.....

tick() changed 4 cells:
..#..
..#..
..#..

getters: width=5 height=3 generation=1 population=3
patternNames() = ["glider", "blinker", "toad", "lwss", "pulsar"]
bindings::console_log on the PC panicked: true
```

> 💡 **Analogy: two people who speak different languages.** Exports are words the robot teaches the browser ("tick", "toggle"); imports are words the browser teaches the robot ("log", "random"). `wasm-bindgen` is the phrasebook both of them carry. On the PC there's no browser to answer, so if the robot says "log!" there, nobody understands, and it stops (panics).

▶ `cargo run -p webassembly --example bindings_on_the_pc`

## 4. Strings and linear memory

Only numbers cross a call, so a string goes through memory: JavaScript encodes it as UTF-8, asks the module to allocate space, copies it in, and passes `(pointer, length)`; the answer comes back the same way. The example does exactly that with a `Vec<u8>` playing the module's memory:

```text
"Ama 👋" in JavaScript: .length = 6 (UTF-16 code units)
"Ama 👋" in Rust:       .len()  = 8 (UTF-8 bytes), 5 chars

copied in:  ptr=8 len=8  memory[0..32] = [0, 0, 0, 0, 0, 0, 0, 0, 65, 109, 97, 32, 240, 159, 145, 139, 0, ...]
greet(8, 8) returned (16, 16)
copied out: "Hello, Ama 👋!"
```

Two lessons from it:

- **Strings and `Vec`s are copied**, both ways. That's cheap for a name, expensive for a 4K image every frame. For big data, let JavaScript look at Rust's memory directly instead (section 5).
- **JavaScript counts string positions in UTF-16 units, Rust in UTF-8 bytes.** They only agree for ASCII. A cursor position from a browser text box must be converted (exercise 4).

> 💡 **Analogy: posting a letter through a letterbox.** Only thin slips of paper with a number on them fit through (i32, f64). To send a long letter, you put it on the **shared shelf** (linear memory) and slip through a note saying "shelf 8, 8 pages long". And the two sides even count pages differently (UTF-16 vs UTF-8), so always say which way you're counting.

▶ `cargo run -p webassembly --example strings_and_memory`

## 5. Passing data: numbers, arrays, structs

Every function in the example carries `#[wasm_bindgen]`, so compiling it is the proof that wasm-bindgen 0.2.130 accepts these types:

```text
Rust type            JavaScript type          example call on the PC
i32, u32, f32, f64   number                   add(2, 3) = 5
bool                 boolean                  is_even(4) = true
u64, i64             BigInt                   big(21) = 42
Option<u32>          number | undefined       first_digit("ab7") = Some(7)
&str, String         string (copied)          shout("hi") = "HI"
&[f32]               Float32Array (copied)    sum([1.5, 2.5]) = 4
Vec<u8>              Uint8Array (copied)      ramp(4) = [0, 1, 2, 3]
&mut [u8]            Uint8Array, copied back  double_in_place([1, 2, 200]) -> [2, 4, 255]
Vec<String>          string[]                 words("a b") = ["a", "b"]
Result<u8, JsError>  u8, or throws Error      parse_percent("42%") = Some(42)
#[wasm_bindgen] struct  class instance        new Point(3, 4).length() = 5
```

What does **not** cross, checked on the PC:

- **Nested collections.** A `Vec<Vec<u8>>` is a compile error, because there's no "array of typed arrays" conversion (exercise 1). Flatten it: one `Vec<u8>`, row after row.

```text
error[E0271]: type mismatch resolving `<Vec<u8> as ErasableGeneric>::Repr == JsValue`
  --> 05_mastery\09_webassembly\exercises\ex01_fix_nested_vec.rs:19:1
   |
19 | #[wasm_bindgen]
   | ^^^^^^^^^^^^^^^ expected `JsValue`, found `Vec<u8>`
   |
   = note: required for `Vec<u8>` to implement `VectorIntoWasmAbi`
```

- **Borrowed return values.** `pub fn shout(text: &str) -> &str` is refused by the macro (no error code, verified in a scratch file): `error: cannot return a borrowed ref with #[wasm_bindgen]`. JavaScript can't hold a Rust borrow; return a `String`.

Three ways to send structs, from fewest to most objects:

| Way | Good for |
|---|---|
| An exported `#[wasm_bindgen] struct` (a class) | one long-lived object with methods, like the `Universe` |
| One flat array of numbers (`Vec<f32>`: x0, y0, x1, y1...) | thousands of small records (exercise 6) |
| A view into Rust's memory: `new Uint8Array(memory.buffer, ptr, len)` | big data read every frame, **no copy at all** (the Game of Life grid) |

An exported struct stays in Rust's memory; JavaScript only holds its **address**, and the garbage collector can't see inside WebAssembly memory. So JavaScript must call **`.free()`** when it's done (the example does the three steps by hand with `Box::into_raw` and `Box::from_raw`):

```text
JavaScript: const a = new Counter("clicks")
    JavaScript holds only the number 0x2082d7cd000
JavaScript: a.free()
    (Rust) dropped "clicks" at count 2

Counters still alive: 1 ("forgotten" leaked: JavaScript dropped the number, but nothing told Rust)
```

> 💡 **Analogy: a cloakroom ticket.** When you hand your coat in (create a Rust object), you get a **ticket with a number** (the pointer). The coat stays in the cloakroom. If you lose the ticket without handing it back (`free()`), the coat hangs there forever. Small things you can just pass through the window (numbers); a pile of papers you photocopy (strings, arrays); and for a huge map, you let the other person **come and look at it on the wall** (a memory view) instead of copying it.

▶ `cargo run -p webassembly --example passing_data` · `cargo run -p webassembly --example handles_and_free`

## 6. Keeping the core testable: `cfg` and a host trait

The Game of Life core needs a random number and a way to log. Instead of calling JavaScript, it asks a trait:

```rust
pub trait Host {
    fn log(&mut self, message: &str);
    fn random(&mut self) -> f64;
}

pub struct JsHost;   // calls console.log / Math.random (panics if called on the PC)
pub struct StdHost;  // println! and xorshift64*
pub struct ScriptedHost; // fixed numbers, for tests

#[cfg(target_arch = "wasm32")]
pub type DefaultHost = JsHost;
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultHost = StdHost;
```

Only that one choice is gated, so `cargo test` and `cargo clippy` on the PC still check every line of the bindings. When something must exist only in WebAssembly, give it a twin for every other target, or calls to it break the PC build (exercise 3):

```text
error[E0425]: cannot find function `now_ms` in this scope
  --> 05_mastery\09_webassembly\exercises\ex03_fix_cfg_gated_import.rs:38:25
   |
38 |     format!("{:.1} ms", now_ms() - started)
   |                         ^^^^^^ not found in this scope
   |
note: found an item that was configured out
  --> 05_mastery\09_webassembly\exercises\ex03_fix_cfg_gated_import.rs:33:4
   |
32 | #[cfg(target_arch = "wasm32")]
   |       ---------------------- the item is gated behind the `wasm32` feature
33 | fn now_ms() -> f64 {
   |    ^^^^^^
```

(The compiler calls `target_arch = "wasm32"` a "feature" there; it means the cfg condition.) Global state is the other trap: a browser runs your module on one thread, but Rust's rules still apply, because the same code compiles for the PC, and WebAssembly can have threads (exercise 2):

```text
error[E0277]: `RefCell<Vec<u32>>` cannot be shared between threads safely
  --> 05_mastery\09_webassembly\exercises\ex02_fix_global_state.rs:25:16
   |
25 | static CLICKS: RefCell<Vec<u32>> = RefCell::new(Vec::new());
   |                ^^^^^^^^^^^^^^^^^ `RefCell<Vec<u32>>` cannot be shared between threads safely
   |
   = help: the trait `Sync` is not implemented for `RefCell<Vec<u32>>`
```

Use `thread_local!`, a `Mutex`, or (best) no global: keep the state in an exported struct that JavaScript holds.

> 💡 **Analogy: a robot with a plug-in phone.** The robot doesn't care *who* answers when it asks "give me a random number": it has a socket (the `Host` trait). In the browser you plug in the JavaScript phone, at home the PC phone, and in tests a toy phone that always says the same thing. One label on the box (`cfg`) says which phone comes in the box.

## 7. `wasm-pack` and the browser page (not run here)

[`wasm-pack`](https://rustwasm.github.io/docs/wasm-pack/) runs the whole pipeline: `cargo build --target wasm32-unknown-unknown`, then `wasm-bindgen` to generate the JavaScript glue, and packages the result in `pkg/`. **None of these commands were run** (the target and tools aren't installed on this machine). From `05_mastery/09_webassembly`:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web          # -> pkg/webassembly.js, pkg/webassembly_bg.wasm, .d.ts, package.json
python -m http.server 8000            # any static server: browsers block modules and .wasm from file://
# open http://localhost:8000/www/
```

Without `wasm-pack`, the same by hand (the CLI **must be the exact version** of the `wasm-bindgen` crate in `Cargo.lock`, 0.2.130 here):

```bash
cargo install wasm-bindgen-cli --version 0.2.130
cargo build -p webassembly --lib --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir 05_mastery/09_webassembly/pkg target/wasm32-unknown-unknown/release/webassembly.wasm
```

`--target web` produces an ES module for a plain `<script type="module">`; `bundler` (the default) is for webpack/Vite, and `nodejs` for Node. The page is [`www/index.html`](www/index.html) and [`www/index.js`](www/index.js) (`pkg/` is in this folder's `.gitignore`):

```js
import init, { Universe, patternNames } from "../pkg/webassembly.js";
const wasm = await init();                       // fetches the .wasm, runs #[wasm_bindgen(start)]
let universe = new Universe(64, 48);
universe.insertPattern("glider", 2, 2);
// every frame: a NEW zero-copy view (an old one dies if memory grows)
const cells = new Uint8Array(wasm.memory.buffer, universe.cellsPtr(), universe.width * universe.height);
// a bad pattern: Rust's Err arrives as a thrown Error
try { const next = Universe.fromPattern(text, 64, 48); universe.free(); universe = next; }
catch (e) { error.textContent = e.message; }
```

`index.js` was only syntax-checked here (`node --check`), never run in a browser.

> 💡 **Analogy: the gift-wrapping counter.** You bring the robot (your crate); `wasm-pack` puts it in the glass box (`.wasm`), writes the instruction card for whoever opens it (the JavaScript glue), and wraps it so it fits under any tree (`pkg/`). The page is the shop window where people see it move.

## 8. Making the `.wasm` small (not run here)

Every byte is downloaded by every visitor, so release builds of wasm are usually tuned for size. In a standalone project's `Cargo.toml` (profiles in a workspace member are ignored, see [03/05](../../03_abstraction/05_cargo_and_crates_io/README.md), so this lesson doesn't set them):

```toml
[profile.release]
opt-level = "z"     # optimise for size ("s" is a little less aggressive)
lto = true          # optimise across crates, removing unused code
codegen-units = 1   # slower build, smaller output
panic = "abort"     # no unwinding machinery
strip = true        # no symbol names
```

Then, as tools to try (**not run here**): `wasm-opt -Oz` from [binaryen](https://github.com/WebAssembly/binaryen) shrinks the module further (`wasm-pack` runs it for release builds by default), and [`twiggy`](https://github.com/rustwasm/twiggy) shows which functions take the space. In code, the usual culprits are `format!` and panics with messages (they pull in the formatting machinery), and big generic functions copied for many types.

> 💡 **Analogy: packing a lunchbox.** You only pack what you'll eat (`lto` drops unused code), squash the sandwich flat (`opt-level = "z"`), and leave the cutlery set at home if a spoon will do (`panic = "abort"`, fewer `format!`s).

## 9. WASI: WebAssembly outside the browser

With a WASI target, ordinary `std` code (args, env, stdin/stdout, the clock, files) works, inside a sandbox: the module can open files only in folders you grant it. The example is a plain word counter with no `wasm-bindgen` at all; on the PC (run):

```text
$ cargo run -p webassembly --example wasi_program -- 05_mastery/09_webassembly/src/lib.rs nope.txt
  lines   words    chars  file
     21     152      884  05_mastery/09_webassembly/src/lib.rs
                   error  nope.txt: The system cannot find the file specified. (os error 2)
```

The same source as WebAssembly (**not run here**: no target, no wasmtime):

```bash
rustup target add wasm32-wasip2
cargo new wasi-demo && cp 05_mastery/09_webassembly/examples/wasi_program.rs wasi-demo/src/main.rs
cd wasi-demo && cargo build --target wasm32-wasip2 --release
wasmtime run --dir=. target/wasm32-wasip2/release/wasi-demo.wasm README.md
```

Without `--dir=.` the module can't open any file at all, and environment variables are empty unless you pass `--env`.

> 💡 **Analogy: a robot in a playroom.** Outside the browser the robot can use a few real toys (files, a clock, a printer), but only in the playroom you open for it (`--dir`). Every other door stays locked.

▶ `cargo run -p webassembly --example wasi_program -- 05_mastery/09_webassembly/README.md`

## The mini project: the Game of Life, for the browser

`src/` is Conway's Game of Life: a pure-Rust core tested on the PC, a thin `#[wasm_bindgen]` layer, and a page in `www/`. On the PC, `main.rs` draws it in the terminal:

```text
$ cargo run -p webassembly
Glider (glider), generations 0 to 4:
  .......   .......   .......   .......   .......
  ..#....   .......   .......   .......   .......
  ...#...   .#.#...   ...#...   ..#....   ...#...
  .###...   ..##...   .#.#...   ...##..   ....#..
  .......   ..#....   ..##...   ..##...   ..###..
  .......   .......   .......   .......   .......
  .......   .......   .......   .......   .......

Pulsar repeats every 3 generations (48 cells alive)

[log] random soup: 32x16, 35% alive
population by generation:  0: 173  20: 86  40: 107  60: 94  80: 105  100: 105
...
```

| File | Lesson topic |
|---|---|
| [`src/life.rs`](src/life.rs) | the core: a flat grid of `#[repr(u8)]` cells (one byte each, readable by JavaScript in place), double-buffered `tick` that never allocates |
| [`src/pattern.rs`](src/pattern.rs) | parsing pasted text into a pattern, with errors (line, column) instead of panics |
| [`src/host.rs`](src/host.rs) | the `Host` trait; `JsHost`, `StdHost`, `ScriptedHost`; `cfg` picks `DefaultHost` |
| [`src/bindings.rs`](src/bindings.rs) | every `#[wasm_bindgen]`: the `Universe` class, `patternNames`, `start`, the `console.log` and `Math.random` imports |
| [`src/main.rs`](src/main.rs) | the PC demo |
| [`www/index.html`](www/index.html), [`www/index.js`](www/index.js) | the browser page: canvas, buttons, zero-copy drawing, `free()` (not run here) |

> 💡 **Analogy: a goldfish bowl you can carry.** The rules of the little world (the core) are the same whether the bowl is on your desk (the terminal) or in a shop window (the browser). Only the glass around it changes.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `#[wasm_bindgen] fn f() -> Vec<Vec<u8>>` | E0271 type mismatch ... `VectorIntoWasmAbi` | Flatten to one `Vec<u8>` plus a width. |
| `#[wasm_bindgen] fn f(s: &str) -> &str` | cannot return a borrowed ref with #[wasm_bindgen] (no code) | Return `String`. |
| `static STATE: RefCell<...>` | E0277 cannot be shared between threads safely | `thread_local!`, a `Mutex`, or state in an exported struct. |
| Called a `#[cfg(target_arch = "wasm32")]` function from shared code | E0425 cannot find function | Add a `#[cfg(not(target_arch = "wasm32"))]` twin, or hide it behind a trait. |
| Called an imported JS function (or `JsError::new`) in a PC test | panic: cannot call wasm-bindgen imported functions on non-wasm targets | Test the pure logic; inject a fake host. |
| `SystemTime::now()` / `Instant::now()` in a browser build | panic: time not implemented on this platform | Import `Date.now()` / `performance.now()` from JavaScript. |
| Forgot `crate-type = ["cdylib", ...]` | no `.wasm` file to use (not run here) | Add `cdylib` (keep `rlib` for tests). |
| `wasm-bindgen-cli` version differs from the crate | the CLI refuses the file (not run here) | Install the exact version from `Cargo.lock`, or use `wasm-pack`. |
| Opened `index.html` straight from disk | the module doesn't load | Serve the folder over HTTP. |
| Kept a `Uint8Array` view across frames | it goes empty after memory grows | Make a new view every time you read. |
| Never called `.free()` on exported objects | memory grows forever | `free()` when done. |
| A JS cursor index used to slice a Rust `&str` | wrong text, or a panic on a char boundary | Convert UTF-16 indexes to byte indexes (exercise 4). |
| Returned a `u64` to JavaScript | it arrives as a `BigInt`, which doesn't mix with `number` | `u32` or `f64` if the value fits. |
| One JS -> wasm call per cell, per frame | slow | Batch: `tickMany`, or read memory directly. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p webassembly --example wasm_module_bytes
cargo run -p webassembly --example bindings_on_the_pc
cargo run -p webassembly --example strings_and_memory
cargo run -p webassembly --example passing_data
cargo run -p webassembly --example handles_and_free
cargo run -p webassembly --example wasi_program -- 05_mastery/09_webassembly/README.md
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_nested_vec` | Make it compile (E0271): a type wasm-bindgen can't pass | `cargo run -p webassembly --features exercises --bin ex01_fix_nested_vec` |
| 2 | `ex02_fix_global_state` | Make it compile (E0277): global state in a wasm module | `cargo run -p webassembly --features exercises --bin ex02_fix_global_state` |
| 3 | `ex03_fix_cfg_gated_import` | Make it compile (E0425): a wasm-only import needs a PC twin | `cargo run -p webassembly --features exercises --bin ex03_fix_cfg_gated_import` |
| 4 | `ex04_utf16_offsets` | JavaScript vs Rust string positions, pass 3 tests | `cargo test -p webassembly --features exercises --bin ex04_utf16_offsets` |
| 5 | `ex05_rgba_filters` | Image filters on RGBA bytes, pass 3 tests | `cargo test -p webassembly --features exercises --bin ex05_rgba_filters` |
| 6 | `ex06_flat_buffers` | Structs as one flat buffer, pass 4 tests | `cargo test -p webassembly --features exercises --bin ex06_flat_buffers` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p webassembly --features solutions --bins`, or `cargo run -p webassembly --features solutions --bin sol01_fix_nested_vec` and so on).

**4. The mini project:**
- `cargo run -p webassembly -- pulsar 3` (or `glider`, `blinker`, `toad`, `lwss`) shows one pattern.
- `cargo test -p webassembly`: 21 unit tests and 1 doc test.
- If you can install the target: section 7, then open the page, click cells, paste a pattern with a mistake in it and read the error.
- Add a `step_back` that keeps the previous generation (you already have two buffers). What has to change in `bindings.rs` and `index.js`?

## Checklist

- [ ] What a `.wasm` module is: sections, a stack machine, linear memory, the sandbox
- [ ] wasm32 targets: `wasm32-unknown-unknown`, `wasm32-wasip1`/`wasip2`, `cdylib`
- [ ] `wasm-bindgen`: exports (functions, classes, getters, `start`) and imports (`js_namespace`)
- [ ] Strings across the boundary: copies, UTF-8 vs UTF-16
- [ ] Passing numbers, arrays and structs; `free()`; zero-copy memory views
- [ ] Keeping the core testable: a host trait and `cfg(target_arch = "wasm32")`
- [ ] `wasm-pack build --target web` and a page that loads it
- [ ] Size optimisation: profile settings, `wasm-opt`, `twiggy`
- [ ] WASI: std programs as sandboxed WebAssembly
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A `.wasm` module is sandboxed stack-machine bytecode with one linear memory; only `i32`, `i64`, `f32` and `f64` cross a call.
- Rust compiles to `wasm32-unknown-unknown` for browsers (no OS: time and files aren't there) and to `wasm32-wasip1`/`wasip2` for WASI hosts.
- `wasm-bindgen` exports Rust functions and structs to JavaScript and imports JavaScript functions into Rust, copying strings and arrays through memory; big data is better read in place.
- Keep the core in plain Rust behind a host trait, so it is tested on the PC; gate only the JavaScript-specific choice with `cfg`.
- `wasm-pack` builds the `pkg/` a page imports; release profiles, `wasm-opt` and fewer `format!`s keep it small.

**In one sentence for a child:** WebAssembly is a glass box that lets your fast little Rust robot work inside any web page, passing notes through a letterbox and sharing one shelf, without ever being able to touch anything it shouldn't.

**Next:** [10 Capstone: a production CLI tool](../10_capstone_cli_tool/README.md)
