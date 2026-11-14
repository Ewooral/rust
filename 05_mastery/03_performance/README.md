<!-- hand-written lesson: gen_course must not overwrite -->
# Performance

**Read alongside:** [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
· Cargo: [Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), [`cargo bench`](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)
· [Criterion.rs user guide](https://bheisler.github.io/criterion.rs/book/) · [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html)
· Reference: [Type layout](https://doc.rust-lang.org/reference/type-layout.html), [the `inline` attribute](https://doc.rust-lang.org/reference/attributes/codegen.html#the-inline-attribute)
· std: [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html), [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) · Crates: [`criterion`](https://docs.rs/criterion), [`rayon`](https://docs.rs/rayon), [`smallvec`](https://docs.rs/smallvec)

Run it: `cargo run -p performance --release`  ·  Test it: `cargo test -p performance`  ·  Benchmark it: `cargo bench -p performance`

Rust is fast by default, but "fast by default" isn't "as fast as it could be". This lesson is about making a program faster **on purpose**: measuring first, building with the right settings, writing benchmarks that tell the truth, finding the slow part with a profiler, and then fixing the usual suspects (allocations, memory layout, missed optimisations, unused cores). After it you'll be able to take a slow Rust program, prove where its time goes, speed it up step by step, and show the numbers that justify every change.

> 💡 **The big picture: a kitchen at rush hour.** The restaurant is slow and everyone has a theory. A good head chef doesn't guess: they put a **stopwatch** on every station (measuring) and fix the slowest one first. Usually the time isn't lost in the cooking. It's lost in **trips to the pantry** (allocations), **walking across the kitchen** for ingredients stored far apart (cache misses), and **re-reading the recipe card** at every step (checks the compiler could have skipped). Only when every cook works smoothly does it pay to **hire more cooks** (parallelism), and only for jobs big enough to share.

The mini project follows exactly that path. One log report is written four times, and each version fixes the biggest cost the measurements found in the one before:

```text
version            time  speed-up  allocations      MB/s
v1_naive       158.90ms      1.0x      2198059        73
v2_borrowed     54.37ms      2.9x       399622       214
v3_fast         22.38ms      7.1x           15       519
v4_parallel      4.32ms     36.8x         8580      2688
```

(One run on the author's machine, 12 threads; yours will differ.)

## 1. Measure before you optimise

Programmers are famously bad at guessing where time goes. So the rule is: **measure, change one thing, measure again.** Timing by hand with `std::time::Instant` works, but it has three traps:

1. **The optimiser deletes work nobody uses.** If the result is thrown away, a release build may never run the code at all. [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html) tells the compiler "assume someone looks at this", so the work stays.
2. **One run is one sample.** Other programs, CPU frequency changes and cold caches all add noise. Run several times and report the **median**.
3. **Guessing which step is slow.** Time each step and fix the biggest number, not the step that *looks* slow.

```rust
let start = Instant::now();
black_box(checksum(black_box(N)));   // black_box on the input AND the output
let kept = start.elapsed();
```

```text
release build

1. result unused:                 0.00ns  (maybe never ran!)
   with std::hint::black_box:     2.65ms

2. five runs of the same thing: [2.52ms, 2.51ms, 2.51ms, 2.52ms, 2.72ms]
   median: 2.60ms  <- report this (or let criterion do it)

3. one job, three steps (200 000 numbers):
   sort       3.64ms
   format     4.37ms
   sum       32.60µs
```

(One run on the author's machine; yours will differ.) The first line is no typo: without `black_box` the loop was deleted. Its first version used a sum of squares, and the optimiser even replaced *that* loop with a formula, so `black_box` didn't help either. The example now uses a hash, where every step depends on the one before. The third block surprises most people: turning numbers into text costs more than sorting them.

Two more rules:
- **Never time a debug build.** `cargo run` builds without optimisations, and the numbers mean nothing. Every timing in this lesson uses `--release`, and the examples print a warning when they're built without it.
- **Know what "fast enough" is.** Optimise when a measurement says something is too slow for its job, not because it could be faster.

> 💡 **Analogy: the stopwatch.** Before changing anything, the head chef times every station for a whole evening. Timing one plate isn't enough, because one plate can be slow for a silly reason (someone dropped it). And a chef who forgets to check that a dish was actually *served* might time an empty plate going out of the door in 0 seconds: that's the deleted loop.

▶ `cargo run -p performance --example measure_first --release` (and once without `--release`)

## 2. Release profiles: build for speed

A [profile](https://doc.rust-lang.org/cargo/reference/profiles.html) is a set of compiler settings ([lesson 03/05](../../03_abstraction/05_cargo_and_crates_io/README.md) introduced them). The settings that matter for speed, with Cargo's defaults:

| Setting | `dev` | `release` | What it does |
|---|---|---|---|
| `opt-level` | `0` | `3` | How hard the optimiser works. `"s"`/`"z"` optimise for size instead. |
| `debug` | `true` | `false` | Debug info: function names and line numbers for debuggers and **profilers**. |
| `lto` | `false` | `false` | Link-time optimisation: `"thin"` or `"fat"` (`true`) optimise **across crates**, so a function in a dependency can be inlined into yours. Slower builds. |
| `codegen-units` | `256` | `16` | The crate is split into this many parts compiled in parallel. `1` gives the optimiser the whole crate at once: faster code, slower builds. |
| `panic` | `"unwind"` | `"unwind"` | `"abort"` skips unwinding on panic: smaller binary, slightly faster, but `catch_unwind` stops working. |
| `debug-assertions`, `overflow-checks` | on | off | `debug_assert!` and integer-overflow panics. |

`cargo bench` uses the `bench` profile, which inherits from `release`, as its output says:

```text
    Finished `bench` profile [optimized] target(s) in 10.32s
```

A tuned setup, in the **workspace root** `Cargo.toml` (in a member's `Cargo.toml` profiles are ignored with a warning, which is why this lesson only shows them):

```toml
[profile.release]
lto = "fat"          # optimise across crates
codegen-units = 1    # optimise the whole crate as one piece
panic = "abort"      # only if nothing relies on catching panics

[profile.profiling]  # a custom profile: cargo build --profile profiling
inherits = "release"
debug = true         # keep function names and line numbers for the profiler
```

- **Debug symbols for profiling.** A release build has no debug info, so a profiler shows addresses or mangled names instead of your functions. Either build with a profile like `profiling` above (the output goes to `target/profiling/`), or set it for one command only with an environment variable: `CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release`. `debug = "line-tables-only"` is a lighter option that keeps names and line numbers but not variable information.
- **`-C target-cpu=native`** (through `RUSTFLAGS`) lets the compiler use every instruction your CPU has. The binary may then crash on an older CPU, so use it only for programs that run where they're built.
- **Measure each setting.** `lto` and `codegen-units = 1` often help a little and sometimes not at all, and they always make builds slower.

> 💡 **Analogy: preparing the kitchen before service.** In the morning (`dev`) cooks practise quickly and taste everything as they go. Before a big evening (`release`) the head chef sharpens every knife and lays out every station (`opt-level = 3`). `lto` is letting the pastry section and the grill **plan the menu together** instead of separately. `codegen-units = 1` is one chef planning the whole menu rather than sixteen planning bits of it: a better plan, but it takes longer. And `debug = true` is **labelling every pot**, so the inspector with the stopwatch knows whose pot is slow.

## 3. Benchmarking with criterion

`Instant` timings are fine for a quick look. To *decide* something, use a benchmark harness. The built-in `#[bench]` needs nightly Rust, so most projects use [criterion](https://bheisler.github.io/criterion.rs/book/). It warms up, chooses how many iterations to run, collects samples, removes outliers, and tells you whether a change is **real or just noise**.

Set it up in three steps:

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.8", default-features = false }   # no HTML plots, fewer dependencies

[[bench]]
name = "pipeline"   # benches/pipeline.rs
harness = false     # criterion brings its own main()
```

```rust
// benches/pipeline.rs (shortened)
use std::hint::black_box;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn compare_versions(c: &mut Criterion) {
    // Build the input ONCE, outside the measurement: only `analyse` is timed.
    let log = logs::generate(20_000, 42);

    let mut group = c.benchmark_group("pipeline");
    group.throughput(Throughput::Bytes(log.len() as u64));   // also report MB/s
    for (name, analyse) in pipeline::VERSIONS {
        group.bench_with_input(BenchmarkId::from_parameter(name), &log, |b, log| {
            b.iter(|| analyse(black_box(log)))
        });
    }
    group.finish();
}

criterion_group! { name = benches; config = quick(); targets = compare_versions }
criterion_main!(benches);
```

- `b.iter(|| …)` is the part that gets timed, many times over. Keep setup outside it.
- Criterion's own `criterion::black_box` is deprecated in 0.8 ("use `std::hint::black_box()` instead"), so use the one from `std`.
- A **group** puts related benchmarks side by side. That's how you compare implementations.
- Criterion's defaults (100 samples, 5 s each) are precise but slow. This course uses `quick()`: 10 samples (the minimum), 0.3 s warm-up, 1 s of measuring, so all of `cargo bench -p performance` took 30 seconds here, build included.
- This lesson's `Cargo.toml` also sets `bench = false` on the library and the main program. Otherwise `cargo bench` would run their (empty) built-in bench harness too, and that harness rejects criterion's options such as `--save-baseline`.

Part of a real run (one run on the author's machine; yours will differ):

```text
pipeline/v1_naive       time:   [26.956 ms 30.248 ms 35.576 ms]
                        thrpt:  [31.113 MiB/s 36.594 MiB/s 41.062 MiB/s]
pipeline/v3_fast        time:   [4.6250 ms 5.1419 ms 6.3121 ms]
                        thrpt:  [175.36 MiB/s 215.27 MiB/s 239.33 MiB/s]
pipeline/v4_parallel    time:   [1.1003 ms 1.2027 ms 1.3568 ms]
                        thrpt:  [815.80 MiB/s 920.30 MiB/s 1006.0 MiB/s]
                 change:
                        time:   [−75.720% −64.557% −45.239%] (p = 0.00 < 0.05)
                        thrpt:  [+82.612% +182.14% +311.86%]
                        Performance has improved.
Found 4 outliers among 10 measurements (40.00%)
```

- The **three numbers** are a confidence interval: the middle one is the best estimate, and the true time is very likely between the outer two. A wide interval means a noisy measurement.
- **`change:`** compares with the previous run, saved under `target/criterion/`. The p-value says how likely the difference is to be luck; above 0.05 criterion says "No change in performance detected".
- **Noise is real.** In that same run, `layout/array_of_structs` was "improved" by 44% and `bounds/indexed` "regressed" by 70%, and **neither had changed**: five other builds were running on the machine. For decisions, benchmark on a quiet machine and run twice.
- **Baselines** compare against a named run instead of the last one: `cargo bench -p performance --bench pipeline -- --save-baseline before`, change the code, then `-- --baseline before`.
- **Filter:** `cargo bench -p performance --bench pipeline -- v3` runs only names containing `v3`.

> 💡 **Analogy: a proper race, not a stopwatch in your hand.** A stopwatch race between two runners on a windy day proves nothing. Criterion is a **real athletics meeting**: a warm-up lap, the same track, many heats, the fastest and slowest heats discussed separately (outliers), and a judge who says "that 2% difference is just the wind" (no change detected) or "that's a new personal best" (improved). Comparing against a **baseline** is comparing with last season's official time, not with your memory of it.

▶ `cargo bench -p performance` (both bench files), or `--bench pipeline` / `--bench kernels`

## 4. Profiling: find where the time goes

A benchmark tells you **how long**. A **profiler** tells you **where**. A *sampling* profiler stops your program thousands of times a second and writes down which function it was in, and its call stack. Functions that show up in many samples are where the time goes.

The usual picture is a **flame graph**: each box is a function, the boxes above it are the functions it called, and **width means time**. Look for wide boxes near the top: that's code that does the work itself, rather than just calling others.

| Tool | Platform | How you use it |
|---|---|---|
| [`perf`](https://perf.wiki.kernel.org/) | Linux | `perf record -g ./program`, then `perf report` |
| [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) | Linux (uses `perf`), Windows (built-in support) | `cargo install flamegraph`, then `cargo flamegraph -p performance --example profiling_target`: writes `flamegraph.svg` (open it in a browser) |
| [samply](https://github.com/mstange/samply) | Linux, Windows | `samply record ./program`: opens the result in the Firefox Profiler |
| Windows Performance Recorder + [Windows Performance Analyzer](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/windows-performance-analyzer) | Windows | record a trace with WPR, open it in WPA, look at "CPU Usage (Sampled)" |
| [Superluminal](https://superluminal.eu/) | Windows (commercial) | start the program from Superluminal; good Rust support |
| Visual Studio | Windows | Debug > Performance Profiler > CPU Usage |

Nothing here needs installing for this course; read the table now and come back when you have a slow program. Whichever tool you use:
1. Profile a **release** build **with debug info** (section 2), or the profile will be full of unnamed addresses.
2. Run long enough to collect samples: a second or more. `profiling_target` runs the naive pipeline for about that long in release.
3. Expect allocator functions (`alloc`, `free`, `HeapAlloc` on Windows), `memcpy` and hashing near the top of slow Rust code. That's what led to versions 2 and 3 of the mini project.

> 💡 **Analogy: the photographer on the ceiling.** Instead of following one cook all evening, a camera on the ceiling takes a photo **every millisecond**. At the end you count: "in 600 of 1 000 photos, someone was walking to the pantry". You didn't have to guess. The flame graph is that count drawn as a picture: the wider the box, the more photos it appears in.

▶ `cargo run -p performance --example profiling_target --release`, then profile `target/release/examples/profiling_target` with one of the tools above

## 5. Allocation awareness

Every `String::from`, `to_string`, `clone()` of a `Vec`, and every time a `Vec`, `String` or `HashMap` **grows**, the program asks the allocator for memory. Each request costs time, and so does freeing it again. In hot code, allocation is often the biggest single cost.

**Count them.** `src/alloc_counter.rs` wraps the system allocator in a [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) that counts every request. A program installs it with one line:

```rust
#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

let (result, stats) = count_allocations(|| do_the_work());
```

Allocation counts are **exact**: they don't depend on the machine's mood like timings do. That makes them perfect for tests, and the mini project's tests and exercises 1, 2, 3 and 6 use them as **allocation budgets**. Real output:

```text
1. Growing a Vec
   push 10 000 items into Vec::new()                13 allocs   131056 bytes
   push 10 000 items into with_capacity(10 000)      1 allocs    40000 bytes
   (0..10 000).collect() (it knows its size)         1 allocs    40000 bytes

2. Building a String
   s = s + &i.to_string(), 1 000 times            1010 allocs    18184 bytes
   write!(s, "{i}") into with_capacity(4 000)        1 allocs     4000 bytes

3. Cloning vs borrowing (1000 words)
   total_len_owned(words.clone())                 1001 allocs    30890 bytes
   total_len_borrowed(&words)                        0 allocs        0 bytes

4. String vs &str parameters
   shout_owned("hello".to_string())                  1 allocs        5 bytes
   shout_borrowed("hello")                           0 allocs        0 bytes
```

The habits that follow from this:
- **Size it up front.** `Vec::with_capacity(n)`, `String::with_capacity(n)`, `HashMap::with_capacity(n)`. An upper bound is fine. `collect()` from an iterator that knows its length already does this.
- **Borrow instead of cloning.** Take `&[T]` and `&str` parameters, not `Vec<T>` and `String`, unless the function really needs to keep the value. A `.clone()` added "to make the borrow checker happy" is often a hidden copy of a whole collection.
- **Format into one buffer.** `write!(out, "{x}")` writes straight into an existing `String`; `x.to_string()` makes a new one every time.
- **Return slices of the input.** The mini project's `parse_line` returns `Entry<'_>` holding `&str` slices of the line: zero allocations per line.

**`Cow`: allocate only when you must.** [`Cow<'a, str>`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) ("clone on write") is either `Borrowed(&'a str)` or `Owned(String)`, decided at run time. A function that *usually* returns its input unchanged can borrow in the common case and allocate only in the rare one:

```rust
fn tabs_to_spaces_cow(line: &str) -> Cow<'_, str> {
    if line.contains('\t') {
        Cow::Owned(line.replace('\t', "    "))
    } else {
        Cow::Borrowed(line)
    }
}
```

Returning a plain `&str` doesn't work when the function sometimes builds a new string, because the new string would die at the end of the function. That's exercise 1:

```text
error[E0515]: cannot return reference to local variable `lower`
  --> 05_mastery\03_performance\exercises\ex01_fix_dangling_cow.rs:23:5
   |
23 |     &lower
   |     ^^^^^^ returns a reference to data owned by the current function
```

**The small-vector idea.** Many collections are almost always tiny: the fields of one line, the tags on a post. A *small vector* such as [`smallvec::SmallVec<[T; N]>`](https://docs.rs/smallvec) keeps the first `N` items **inside itself** (on the stack, or wherever the struct lives) and only moves to the heap ("spills") when there are more. It's a trade: the value gets bigger, and every access checks which mode it's in. Measure before adopting it.

```text
Cow<str>: normalise 1 000 lines (10 contain a tab)
  always a new String:         1010 allocations
  Cow, borrow when unchanged:    20 allocations
  "no tabs here" came back Borrowed

SmallVec<[usize; 8]>: comma positions
  a,b,c                  2 commas  spilled: false allocations: 0
  1,2,3,4,5,6,7,8,9,10   9 commas  spilled: true  allocations: 1
  "a,b,c" into a plain Vec instead: 1 allocation
```

> 💡 **Analogy: trips to the pantry.** Every allocation is a cook **walking to the pantry**. `with_capacity` is taking a big enough bowl the first time instead of going back for a bigger one again and again (13 trips for 10 000 items!). Cloning is photocopying the whole recipe book to read one page; borrowing is reading it where it lies. `Cow` is "**only fetch a new plate if this one needs changing**". A `SmallVec` is a **little tray on your workbench**: the first eight spoons go on it, and you only walk to the pantry if you need a ninth.

▶ `cargo run -p performance --example allocations` and `--example cow_and_smallvec` (the counts are the same in debug and release)

## 6. Cache-friendly data layout

The CPU doesn't read memory one value at a time. It fetches 64-byte **cache lines**, and reading from its caches is many times faster than going out to main memory. So the way you **arrange** data decides how much of each fetch is useful.

**Array of structs (AoS)** is the natural `Vec<Particle>`: each particle's 8 fields side by side, 32 bytes each. Counting the heavy particles reads only `mass`, but every cache line also brings in seven other fields that are never looked at. **Struct of arrays (SoA)** stores one `Vec` per field, so a loop over masses reads nothing but masses, and the compiler can compare several per instruction (SIMD).

```rust
struct Particle { x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32, mass: f32, charge: f32 }
let aos: Vec<Particle> = …;                       // 32 MB for a million

struct Particles { x: Vec<f32>, /* … */ mass: Vec<f32>, charge: Vec<f32> }
let soa = Particles::from_aos(&aos);              // the masses alone: 4 MB
```

```text
Padding: the same three fields (u8, u64, u8)
  default layout: 16 bytes
  #[repr(C)]:     24 bytes

count the particles heavier than 4.0:
  array of structs     1.86ms
  struct of arrays   252.50µs
```

(One run on the author's machine; yours will differ. The `layout` group in `cargo bench --bench kernels` measures the same thing.)

- SoA wins when loops touch **a few fields of many items**. AoS is fine, and simpler, when you usually use all of one item's fields together. Exercise 4 builds an SoA "world" for a game.
- **Padding:** a `u64` must start at a multiple of 8 bytes, so a `u8` in front of it wastes 7 bytes. By default Rust **reorders fields** to waste less, which is why the default layout above is 16 bytes. `#[repr(C)]` keeps your order (needed when talking to C), and the struct grows to 24.
- Smaller items mean more of them per cache line: `u32` instead of `u64` when the range allows, an index instead of a pointer, an enum instead of a `String`.

> 💡 **Analogy: shelves in the kitchen.** AoS is keeping **each recipe's ingredients in their own box**: to collect all the salt for tonight you open every box and carry the whole box over, just for a pinch. SoA is a **spice rack**: all the salt in one row, so you sweep along it in one go. Padding is packing a lunchbox: put the big sandwich in first, then fill the gaps with the grapes, and the box closes.

▶ `cargo run -p performance --example data_layout --release`

## 7. Iterators vs loops: zero-cost abstractions

"Zero-cost abstraction" means a high-level feature compiles to code as fast as what you'd write by hand. Iterators are the classic example: `filter`, `map` and `sum` are small functions whose closures are **inlined**, and in a release build the chain becomes one tight loop.

```rust
values.iter().filter(|&&v| v.is_multiple_of(2)).map(|&v| v * v).sum()
```

```text
while + index      530.30µs   values[i], i += 1
for over items     468.80µs   for &v in values
iterator chain     529.50µs   filter(..).map(..).sum()
```

(One run on the author's machine; yours will differ. The `loops` group in `cargo bench --bench kernels` gives the same verdict: within noise of each other.) In a **debug** build the iterator chain was the slowest of the three (12.9 ms against 4.5 ms for the index loop), because nothing is inlined and every adapter is a real function call. That's one more reason never to judge speed from a debug build.

So write the clearest version. Iterators are often *faster* in practice, because they have no indexes to check (section 9) and make it obvious to the compiler that items are independent.

> 💡 **Analogy: a conveyor belt.** You can describe the job as "walk to tray 1, pick up the carrot, walk back, peel it; walk to tray 2…" or as "**put the carrots on the belt: wash, peel, chop**". The second is how you'd explain it to a person, and the factory (the compiler) builds the same machine either way. In the practice kitchen (debug) the belt is slower, because each station stops and waits; on the real night it runs as one smooth line.

▶ `cargo run -p performance --example iterators_vs_loops --release`

## 8. `#[inline]`: let the optimiser see inside a call

**Inlining** pastes a function's body into the place that calls it. That saves the call itself, but the real win is that the optimiser can then optimise the body *together with* the loop around it, for example with SIMD.

- **Within one crate** the compiler inlines small functions on its own. You rarely need an attribute.
- **Across crates**, a normal (non-generic) function's body is not available to the caller's crate, so it can't be inlined unless LTO is on. `#[inline]` makes the body available. That's why libraries put it on small, hot functions; this lesson's library puts it on `parse_line`, `Totals::record` and `FnvHasher::write`. Generic functions are compiled in the caller's crate anyway.
- `#[inline(always)]` and `#[inline(never)]` are stronger hints. Use them for experiments like this one, rarely in real code: inlining big functions everywhere makes code larger, and can make it slower.

```rust
#[inline(never)]
fn scale_never(x: f32) -> f32 { x * 1.5 + 2.0 }
```

```text
1 000 000 calls of a tiny function:
  #[inline(never)]    842.70µs
  #[inline(always)]   255.90µs
```

(One run on the author's machine; yours will differ.) Writing this example went wrong in an instructive way: the first version never read `out` afterwards, so the optimiser deleted both loops and both timings were about 0. A `black_box(&out)` after each loop fixed it. That's trap 1 from section 1 again.

> 💡 **Analogy: keeping the tool in your hand.** Calling a function is putting the knife down, walking to the drawer, using the special gadget, and walking back. Inlining is **having the gadget in your hand already**: no walking, and since you can see the whole job, you can chop four carrots at once. But if every cook carried every gadget, nobody could move: that's why the compiler, not you, usually decides.

▶ `cargo run -p performance --example inlining --release`

## 9. Bounds checks and how they disappear

Every `v[i]` checks that `i < v.len()` and panics if not. That check is what makes buffer overflows impossible in safe Rust. It's usually cheap (a comparison the CPU predicts correctly), and the compiler **removes it** whenever it can prove the index is in range. You help it prove that by:

- **using iterators** (`iter()`, `zip`, `windows`, `chunks`): no indexes, nothing to check;
- **re-slicing to the same length first**: `let (b, out) = (&b[..n], &mut out[..n]);` does one check before the loop, and then every `i < n` is known to be in range;
- **asserting up front**: `assert!(b.len() >= a.len());` before the loop has the same effect;
- **`get(i)`** when an index might really be out of range: you get an `Option` instead of a panic.

(`get_unchecked` skips the check, but it's `unsafe`: an out-of-range index is then undefined behaviour. It's a last resort for a loop a profiler has proven hot, after the options above have failed.)

An out-of-range constant index into an **array** is caught while compiling. This error has no code; it's the deny-by-default `unconditional_panic` lint, found while writing the example:

```text
error: this operation will panic at runtime
 --> arr.rs:3:13
  |
3 |     let x = v[7];
  |             ^^^^ index out of bounds: the length is 3 but the index is 7
  |
  = note: `#[deny(unconditional_panic)]` on by default
```

The honest result of this lesson's measurement:

```text
out[i] = a[i] + b[i] for 4096 floats, 1 000 times:
  index all three slices              1.00ms
  re-slice to len first, then [i]     1.05ms
  zip, no indexes at all            959.00µs
```

(One run on the author's machine; yours will differ.) **No difference.** The optimiser already moved the checks out of this simple loop, and all three versions use SIMD. While writing the lesson, five other loop shapes (a dot product, a sliding window, a reversed copy, a gather through an index table) were tried too, and modern Rust handled every one of them. So this is section 1's rule again: **measure before you contort your code**. Rewrite a loop for bounds checks only when a profiler shows it's hot. To see what the compiler did, read the assembly with [Compiler Explorer](https://godbolt.org) or the `cargo-show-asm` tool.

> 💡 **Analogy: checking the recipe card.** A careful cook checks "is there really a step 7?" before every step. A smart cook counts the steps **once** at the start ("12 steps, and I'm doing 1 to 12"), and then never checks again. Re-slicing is counting once; iterators are following the card line by line, so there's nothing to count. Most cooks in Rust's kitchen are already smart: check whether yours is before you teach it.

▶ `cargo run -p performance --example bounds_checks --release`

## 10. Parallelism with rayon

[rayon](https://docs.rs/rayon) (introduced in [04/01 Fearless concurrency](../../04_systems_level/01_fearless_concurrency/README.md)) turns `iter()` into `par_iter()` and spreads the work over every core. The performance questions are **when** it pays and **how** to combine results without sharing:

```text
rayon will use 12 threads

heavy work: Collatz steps for 200 000 numbers
  iter()        32.58ms
  par_iter()     4.79ms  (6.8x faster)

tiny work: the sum of 1 000 numbers
  iter()       200.00ns
  par_iter()    47.00µs  <- usually SLOWER: the overhead wins
```

(One run on the author's machine; yours will differ.)

- **Enough work per item, or enough items.** Splitting the job, waking the threads and joining the results costs microseconds. For a 200 ns job that's a 200x slowdown.
- **Don't share, combine.** Updating one shared variable from many threads would be a data race, and rayon's `for_each` takes an `Fn` closure, so the compiler refuses (exercise 5):

```text
error[E0594]: cannot assign to `total`, as it is a captured variable in a `Fn` closure
  --> 05_mastery\03_performance\exercises\ex05_fix_parallel_sum.rs:26:35
   |
25 |     let mut total = 0;
   |         --------- `total` declared here, outside the closure
26 |     files.par_iter().for_each(|f| total += f.bytes);
   |                      -------------^^^^^^^^^^^^^^^^-
   |                      |        |   |
   |                      |        |   cannot assign
   |                      |        in this closure
   |                      expects `Fn` instead of `FnMut`
```

  The fix is not a `Mutex` (every thread would queue for it) but `map(…).sum()`: each thread sums its own share, then rayon adds the partial sums. For bigger results, `fold` gives each thread its own accumulator and `reduce` merges them. That's version 4 of the mini project:

```rust
input
    .par_lines()
    .fold(|| Partial::with_capacity(EXPECTED_PATHS), |mut partial, line| {
        partial.add_line(line);
        partial
    })
    .reduce(Partial::default, Partial::merge)
    .finish()
```

- **Parallelise last.** Making each thread efficient first (versions 2 and 3) multiplies with the cores; parallelising wasteful code just spreads the waste. Note that version 4 allocates more than version 3: `fold` makes one `Partial` per piece of work rayon splits off, not one per thread.

> 💡 **Analogy: hiring more cooks.** Ten cooks peel a mountain of potatoes ten times faster. But for **one** potato, explaining the job to ten cooks takes longer than peeling it yourself. And you don't make them share one bowl, fighting over it: each peels into **their own bowl** (`fold`), and at the end the bowls are tipped together (`reduce`).

▶ `cargo run -p performance --example rayon_parallel --release`

## The mini project: a log-analysis pipeline, one measured step at a time

`src/` analyses a web server's access log (generated by `src/logs.rs` with a seeded generator, so every run sees the same data) and reports, per path, the requests, errors, total and maximum latency:

```text
2026-11-14T10:00:07Z 203.0.113.42 GET /api/orders 200 37
```

| Version | The change | Why (what the measurements said) |
|---|---|---|
| `analyse_naive` | copies every line and field into `String`s, `HashMap<String, PathStats>` | the starting point: about 11 allocations per line |
| `analyse_borrowed` | `&str` slices of the input everywhere | the profile was full of `alloc`/`free`/`memcpy` |
| `analyse_fast` | no `Vec` per line, a pre-sized map, the FNV hasher | the last allocation per line; SipHash (the default, built to resist attackers) is slow for short trusted keys |
| `analyse_parallel` | version 3 with rayon `fold` + `reduce` | each core was now efficient, so more cores pay |

```bash
cargo run -p performance --release                  # 200 000 lines: the table at the top
cargo run -p performance --release -- 1000000       # a million lines
cargo run -p performance                            # debug: 10 000 lines, and a warning
cargo bench -p performance --bench pipeline         # the careful measurement
cargo test -p performance                           # 20 unit tests + 1 doc test
```

```text
All four versions agree. The 5 paths that cost the most time:

200000 lines, 395 malformed, 13 paths
path                   requests  errors   total ms   avg ms  max ms
/reports/monthly          12432     351   13447250   1081.7   10338
/api/checkout             12579     359    3811755    303.0    2875
/api/orders               24872     754    1864562     75.0    1035
/api/products/search      12625     350    1835204    145.4    1380
/login                    25251     759     728183     28.8     460
```

What makes it trustworthy:
- **Correctness first.** The program and the tests check that **all four versions give the same report**: on a hand-written sample with malformed lines, on a generated log, and on empty input. A faster wrong answer is worthless.
- **Allocation budgets in tests.** `naive_allocates_per_line_but_fast_does_not` asserts more than 7 000 allocations for 1 000 lines in version 1 and at most 20 for version 3. Unlike a timing test, it can never flake.
- **Benchmarks prove each step:** `benches/pipeline.rs` compares the versions, and `benches/kernels.rs` the data-layout, loop and bounds-check micro-benchmarks.
- **Noise is shown honestly.** Running the program twice while other builds were running gave 523 ms and 966 ms for version 1; on a quiet machine it took 159 ms. Relative speed-ups survive noise much better than absolute times do.

> 💡 **Analogy: the kitchen makeover, one change at a time.** Version 1 is the cook who fetches every ingredient separately from the pantry and photocopies the recipe at every step. Version 2 reads the recipe where it lies. Version 3 lays everything out before service and uses the quick scales instead of the bank vault's. Version 4 hires eleven more cooks, now that each one knows how to work. And after every change the head chef **tastes the dish** (the tests) and **checks the stopwatch** (the benchmarks).

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Timed a `cargo run` (debug) build | numbers 3-50x too slow, wrong winner | `--release`, or `cargo bench` |
| Timed code whose result is unused | ~0 ns: the work was deleted | `std::hint::black_box` on the input and the result |
| `criterion::black_box(x)` | deprecated warning in criterion 0.8 | `std::hint::black_box(x)` |
| One timing run | noise taken for a result | median of several runs, or criterion |
| `[profile.release]` in a member's `Cargo.toml` | warning: profiles for the non root package will be ignored | put it in the workspace root `Cargo.toml` |
| Profiled a release build without debug info | addresses instead of function names | `debug = true` in a profiling profile, or `CARGO_PROFILE_RELEASE_DEBUG=true` |
| `cargo bench -- --save-baseline x` with a lib target | `error: Unrecognized option: 'save-baseline'` from the built-in harness | `bench = false` on `[lib]`/`[[bin]]`, or `--bench name` |
| Returned `&str` of a `String` made inside the function | E0515 | return the `String`, or `Cow<'_, str>` |
| Methods taking `self` called twice | E0382 use of moved value | `&self`; return `&[T]` instead of `Vec<T>` |
| `.clone()` to silence the borrow checker | compiles, copies the data every time | borrow: `&T`, `&[T]`, `&str` |
| `Vec::new()` + push in a loop of known length | ~log2(n) reallocations | `with_capacity(n)` or `collect()` |
| `s = s + &x.to_string()` in a loop | an allocation per piece | `write!(s, "{x}")` into one `String::with_capacity` |
| Mutated a captured variable in `par_iter().for_each` | E0594 / E0596 | `map(..).sum()`, `filter(..).collect()`, `fold` + `reduce` |
| `par_iter()` on tiny work | slower than sequential | stay sequential below ~a millisecond of work |
| Unsafe `get_unchecked` "for speed" | undefined behaviour if wrong, often no gain | iterators, re-slicing, `assert!` first; measure |
| `#[inline(always)]` everywhere | bigger and sometimes slower code | let the compiler decide; `#[inline]` on small cross-crate functions |
| `v[7]` on a 3-element array | error: this operation will panic at runtime (no code) | `v.get(7)` |

## Practise

**1. Run the examples** (timings need `--release`; the allocation examples don't):

```bash
cargo run -p performance --example measure_first --release
cargo run -p performance --example allocations
cargo run -p performance --example cow_and_smallvec
cargo run -p performance --example data_layout --release
cargo run -p performance --example iterators_vs_loops --release
cargo run -p performance --example inlining --release
cargo run -p performance --example bounds_checks --release
cargo run -p performance --example rayon_parallel --release
cargo run -p performance --example profiling_target --release
cargo bench -p performance
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_dangling_cow` | Make it compile (E0515) with `Cow`, zero allocations in the common case | `cargo test -p performance --features exercises --bin ex01_fix_dangling_cow` |
| 2 | `ex02_preallocate` | Make it pass: allocation budgets (9 and 109 allocations, budget 1) | `cargo test -p performance --features exercises --bin ex02_preallocate` |
| 3 | `ex03_borrow_dont_clone` | Make it compile (E0382) by borrowing, not cloning | `cargo test -p performance --features exercises --bin ex03_borrow_dont_clone` |
| 4 | `ex04_struct_of_arrays` | Implement an SoA game world, pass 4 tests | `cargo test -p performance --features exercises --bin ex04_struct_of_arrays` |
| 5 | `ex05_fix_parallel_sum` | Make it compile (E0594, E0596) with rayon `sum`/`collect` | `cargo test -p performance --features exercises --bin ex05_fix_parallel_sum` |
| 6 | `ex06_fast_word_count` | Implement a top-words counter within 3 allocations, pass 3 tests | `cargo test -p performance --features exercises --bin ex06_fast_word_count` |

Exercise 2 starts like this (real output):

```text
---- tests::squares_allocate_once stdout ----
thread 'tests::squares_allocate_once' (4212) panicked at 05_mastery\03_performance\exercises\ex02_preallocate.rs:66:9:
assertion `left == right` failed: squares(1_000) is over budget
  left: 9
 right: 1
---- tests::csv_allocates_once stdout ----
thread 'tests::csv_allocates_once' (24256) panicked at 05_mastery\03_performance\exercises\ex02_preallocate.rs:83:9:
assertion `left == right` failed: to_csv of 100 rows is over budget
  left: 109
 right: 1
```

and exercise 3 like this:

```text
error[E0382]: use of moved value: `playlist`
  --> 05_mastery\03_performance\exercises\ex03_borrow_dont_clone.rs:48:31
   |
46 |     let playlist = sample();
   |         -------- move occurs because `playlist` has type `Playlist`, which does not implement the `Copy` trait
47 |     println!("{} songs", playlist.titles().len());
   |                                   -------- `playlist` moved due to this method call
48 |     println!("{} characters", playlist.total_chars());
   |                               ^^^^^^^^ value used here after move
   |
note: `Playlist::titles` takes ownership of the receiver `self`, which moves `playlist`
  --> 05_mastery\03_performance\exercises\ex03_borrow_dont_clone.rs:27:15
   |
27 |     fn titles(self) -> Vec<String> {
   |               ^^^^
```

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p performance --features solutions --bins`).

**4. The mini project:** run it in debug and in release, then benchmark it. Then try a change of your own and prove it: save a baseline (`cargo bench -p performance --bench pipeline -- --save-baseline before`), make version 3's `parse_status` parse the three digits by hand instead of with `str::parse`, run `-- --baseline before`, and keep the change only if criterion says it improved and the tests still pass.

## Checklist

- [ ] Measure before optimising: `Instant`, `black_box`, medians, release builds only
- [ ] Release profiles: `opt-level`, `lto`, `codegen-units`, `panic`, debug symbols for profiling
- [ ] Benchmarking with criterion: `benches/`, `harness = false`, groups, throughput, change detection, baselines
- [ ] Profiling tools: sampling and flame graphs; `perf`, cargo-flamegraph, samply, WPR/WPA, Superluminal
- [ ] Allocation awareness: counting allocations, `with_capacity`, borrowing instead of cloning, `&str` vs `String`, `Cow`, small vectors
- [ ] Cache-friendly data layout: array of structs vs struct of arrays, padding and `#[repr(C)]`
- [ ] Iterators vs loops: zero-cost abstractions, and why debug builds lie
- [ ] `#[inline]`: what inlining enables, and when the attribute matters
- [ ] Bounds-check elimination: iterators, re-slicing, `assert!`, and measuring first
- [ ] Parallelism with rayon: when it pays, `sum`/`collect`/`fold` + `reduce` instead of shared state
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- **Measure first**, in release, with `black_box` and several runs; fix the biggest number, not the suspected one.
- **Profiles** set the optimiser's effort (`opt-level`, `lto`, `codegen-units`, `panic`) in the workspace root; keep debug info when profiling.
- **Criterion** benchmarks live in `benches/` with `harness = false`, compare implementations in groups, and tell real changes from noise.
- A **profiler** shows where the time goes; a flame graph's wide boxes are the hot code.
- The usual wins: **fewer allocations** (capacity, borrowing, `Cow`), **cache-friendly layouts** (SoA for loops over one field), **iterators** (zero-cost, no bounds checks), and **rayon** for big enough work, combining results instead of sharing them.
- Allocation counts are exact, so they make benchmarks-as-tests that never flake.

**In one sentence for a child:** to make a slow kitchen fast, first time every station with a stopwatch, then stop the cooks running to the pantry and across the room, and only then hire more cooks for the big jobs.

**Next:** [04 API design](../04_api_design/README.md)
