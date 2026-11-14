<!-- hand-written lesson: gen_course must not overwrite -->
# Observability

**Read alongside:** [`tracing`](https://docs.rs/tracing) and [`tracing-subscriber`](https://docs.rs/tracing-subscriber) docs
· [`EnvFilter` directives](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
· [Tokio: getting started with tracing](https://tokio.rs/tokio/topics/tracing)
· [`log`](https://docs.rs/log) · [OpenTelemetry for Rust](https://opentelemetry.io/docs/languages/rust/), [`tracing-opentelemetry`](https://docs.rs/tracing-opentelemetry)

Run it: `cargo run -p observability`  ·  Test it: `cargo test -p observability`

A program in production is a black box. When a customer says "my order failed at 14:02", `println!("here")` won't help you. **Observability** is building the program so it can answer questions you haven't thought of yet: what happened, to which order, in what order, how long it took, and how often. In Rust the standard tool is [`tracing`](https://docs.rs/tracing). After this lesson you'll be able to log structured **events** at the right **levels**, group them into **spans** (also across `.await`), choose what you see with `RUST_LOG`, emit JSON for log collectors, collect logs from crates that use `log`, build your own **layers** (metrics), know where OpenTelemetry fits, and **test** your log output.

> 💡 **The big picture: parcel tracking.** Every parcel gets a **tracking number**, and every depot that touches it **scans** it: "arrived Leeds 09:14", "out for delivery", "damaged!". The scans are **events**; the parcel's whole journey, with its tracking number printed on every scan, is a **span**. "Damaged!" is more urgent than "arrived" (**levels**). You choose which notifications you want (**filters**). The same scans feed the website for people, a barcode feed for machines (**JSON**), and the big board in the warehouse that counts parcels per hour (**metrics**). Nobody follows a parcel around with a notebook: it all comes from the scans.

## 1. Why `tracing`, not `println!`

| `println!` | `tracing` |
|---|---|
| always prints, everywhere | the **application** decides what's recorded, where it goes, and how it looks |
| a sentence | a sentence **plus typed fields** (`order.id=7`) you can search and filter |
| no context | **spans** say which request, job or user each line belongs to, even with 100 tasks interleaved |
| one output | many outputs at once: terminal text, JSON, metrics, OpenTelemetry |

**Libraries emit, applications subscribe.** A library only calls `info!`, `#[instrument]` and so on. The program's `main` installs a **subscriber** once, which collects from every crate. This lesson's mini project does that in `src/telemetry.rs`.

> 💡 **Analogy:** `println!` is shouting in the depot: everyone hears it, nobody can search it later, and nobody knows which parcel you meant. `tracing` is **scanning the barcode**: the scan is recorded with the tracking number, and the head office decides who sees what.

## 2. Events and levels

An **event** is something that happened at a moment: `info!("job done")`. Each has one of five **levels**:

| Level | For | Example |
|---|---|---|
| `error!` | something failed and needs attention | a job failed for good |
| `warn!` | something went wrong but was handled | a retry |
| `info!` | the normal story of the program | batch started, job done |
| `debug!` | details for the developer | each attempt |
| `trace!` | every little step | raw results |

Without a subscriber, events go nowhere (and cost almost nothing). `tracing_subscriber::fmt()` prints them; its default maximum level is INFO (`DEFAULT_MAX_LEVEL` in tracing-subscriber's source):

```text
--- no subscriber:
--- fmt() with its default max level (INFO):
 INFO events_and_levels: info: the normal story of the program
 WARN events_and_levels: warn: something odd, but handled
ERROR events_and_levels: error: something failed
--- with_max_level(Level::TRACE):
TRACE events_and_levels: trace: every little step
DEBUG events_and_levels: debug: details for the developer
…
```

- `fmt().init()` installs the subscriber **globally**, for every thread, once per program. `tracing::subscriber::with_default(subscriber, || …)` uses it only inside a closure, on this thread: that's what tests and examples use.
- If building a field is expensive, ask first: `if tracing::enabled!(Level::DEBUG) { … }`.
- **Choose levels for the reader at 3 a.m.**: ERROR wakes someone up, so it's only for failures that need a person.

> 💡 **Analogy:** the depot's scanner has five buttons, from "parcel moved one metre" (TRACE) to "parcel on fire!" (ERROR). Head office decides which buttons ring a bell in their office; the scanner just records the press.

▶ `cargo run -p observability --example events_and_levels`

## 3. Fields: structured values

Fields turn a log line into **data**: `name = value` pairs a log tool can search, filter and add up.

```rust
info!(user_id = 42, admin = false, "user logged in");
info!(user.name = "ama", user.country = "GH", "profile loaded"); // dotted names group fields
info!(path, "request received");          // a variable on its own: its name is the field name
info!(?order, "with ? (Debug)");          // your types: ? = Debug, % = Display
info!(%order, "with % (Display)");
info!(order.id = order.id, order.items = order.items.len(), "chosen fields");
```

```text
 INFO user logged in user_id=42 admin=false
 INFO profile loaded user.name="ama" user.country="GH"
 INFO request received path="/api/orders"
 INFO with ? (Debug) order=Order { id: 7, items: ["tea", "scone"] }
 INFO with % (Display) order=order #7 (2 items)
 INFO chosen fields order.id=7 order.items=2
```

Only types that implement `tracing::Value` (numbers, bools, strings, errors and a few more) can be a field as they are. Your own struct can't, which is exercise 1:

```text
error[E0277]: the trait bound `Order: Value` is not satisfied
  --> 05_mastery\06_observability\exercises\ex01_fix_field_value.rs:24:5
   |
24 |     info!(order, "order placed");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `Value` is not implemented for `Order`
  --> 05_mastery\06_observability\exercises\ex01_fix_field_value.rs:18:1
   |
18 | struct Order {
   | ^^^^^^^^^^^^
```

`?order` compiles, but the order becomes one opaque string. Separate fields (`order.id`, `order.total_pence`) can be queried: "every failed order over £10".

**Recording later.** A span field can start as `field::Empty` and be filled with `span.record("status", 200)`. Record each field **once**: the text formatter shows every value it was given. The mini project's first draft recorded `attempts` on every retry and printed `job{… attempts=1 attempts=2}`; now it records once, when the job ends.

> 💡 **Analogy:** a scan isn't a sentence on a postcard, it's a **form with boxes**: tracking number, depot, weight. A computer can add up the weight boxes; it can't add up "a heavy-ish parcel". `?` is stapling a photo of the parcel to the form: you can look at it, but you can't sort by it.

▶ `cargo run -p observability --example fields_and_values`

## 4. Spans and `#[instrument]`

A **span** is a stretch of time with a name and fields: a request, a job, a database query. While a span is **entered**, every event belongs to it, and spans nest:

```rust
let checkout = info_span!("checkout", order_id = 7);
let entered = checkout.enter();       // events from here on are inside `checkout`
info!("started");
drop(entered);                        // leave it again
checkout.in_scope(|| info!("back inside for a moment"));

#[instrument]                         // a span for the whole function, fields = the arguments
fn add_to_cart(user: &str, item: &str, quantity: u32) -> u32 { … }

#[instrument(skip(user, password), fields(user = %user.name), ret, err)]
fn log_in(user: &User, password: &str) -> Result<u32, String> { … }

#[instrument(level = "debug")]        // a span that only exists when DEBUG is on
fn price_with_tax(pence: u32) -> u32 { … }
```

```text
 INFO checkout{order_id=7}: started
 INFO checkout{order_id=7}:add_to_cart{user="ama" item="tea" quantity=2}: added
DEBUG checkout{order_id=7}:price_with_tax{pence=500}: adding 20% VAT
 INFO checkout{order_id=7}: priced total=600
 INFO log_in{user=ama}: return=42
ERROR log_in{user=ama}: error=password too short
```

(With `.with_span_events(FmtSpan::CLOSE)`, the example also prints a `close` line as each span ends; with timestamps switched on, those lines carry `time.busy` and `time.idle`.)

| `#[instrument(...)]` option | Does |
|---|---|
| `skip(a, b)` / `skip_all` | don't record these arguments (secrets, big values, `self`) |
| `fields(k = v, k2 = %x)` | add fields; `field::Empty` to record later |
| `name = "job"`, `level = "debug"` | the span's name and level |
| `ret`, `err` | log the return value, or an `Err` as an ERROR event |

`#[instrument]` records every argument with `Debug`. An argument type without `Debug` doesn't compile, and the compiler's suggestion is dangerous (exercise 2):

```text
error[E0277]: `Password` doesn't implement `Debug`
   --> 05_mastery\06_observability\exercises\ex02_fix_instrument_secret.rs:16:1
    |
 16 | #[instrument]
    | ^^^^^^^^^^^^^ the trait `Debug` is not implemented for `Password`
…
help: consider annotating `Password` with `#[derive(Debug)]`
```

Deriving `Debug` would compile and write every password into the logs. `skip(password)` is the fix, and the exercise's test checks the secret never appears.

> 💡 **Analogy:** a span is the **parcel's journey**, and its tracking number is printed on every scan made during the journey. A parcel inside a container is a span inside a span: the scan says `container 12 > parcel 7: arrived`. `skip(password)` is not photocopying the letter inside the parcel.

▶ `cargo run -p observability --example spans_and_instrument`

## 5. Subscribers and layers

A subscriber is usually a **registry** (it keeps track of spans) with **layers** stacked on it. Every layer sees every span and event:

```rust
let subscriber = tracing_subscriber::registry()
    .with(fmt::layer().with_filter(LevelFilter::INFO))   // text, INFO and above
    .with(metrics.layer())                               // counts everything
    .with(SpanCloseLogger);                              // our own layer
```

- **A filter on one layer** (`with_filter`) affects only that layer. **A filter on the registry** (`.with(filter)` as a layer) affects them all. In the example, the text output hides DEBUG, but the metrics layer still counts it:

```text
 INFO job{id=1}: ok outcome="succeeded"
    [SpanCloseLogger] span `job` closed
 WARN job{id=2}: bad request outcome="failed"
    [SpanCloseLogger] span `job` closed
…
metrics: events {"DEBUG": 3, "INFO": 2, "WARN": 1}
         outcomes {"failed": 1, "succeeded": 2}
```

- **Writing a layer:** implement `tracing_subscriber::Layer<S>` and override the callbacks you need: `on_event`, `on_new_span`, `on_close`, `on_record`. Read fields with a `Visit`or (`record_str`, `record_u64`, `record_debug`, …), and keep per-span data in the span's **extensions** (`span.extensions_mut().insert(..)`). `src/metrics.rs` times every `job` span that way. Exercise 6 writes a counting layer.
- `Option<Layer>` is a layer too (`None` does nothing). That's how `src/telemetry.rs` switches between text and JSON.
- **Never panic in a layer.** While writing exercise 6, its first draft used `todo!()` in `on_event` and `on_close`; the second panic happened while the first was unwinding, and the test process aborted with `STATUS_STACK_BUFFER_OVERRUN` instead of reporting a failure.

> 💡 **Analogy:** each scan is shown to everyone in a row of desks: the **website desk** writes it up for customers (fmt), the **counting desk** adds it to the board (metrics), the **security desk** only cares about damaged parcels (a filtered layer). Each desk decides for itself what to do with the scan.

▶ `cargo run -p observability --example layers_and_metrics`

## 6. `EnvFilter` and `RUST_LOG`

[`EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) (the `env-filter` feature) chooses what to see with **directives**, usually read from the `RUST_LOG` environment variable, so you change it without recompiling:

| Directive | Means |
|---|---|
| `info` | INFO and more severe, from everywhere |
| `warn,my_app::db=debug` | WARN everywhere, but DEBUG from the `my_app::db` module |
| `info,hyper=off` | silence one noisy dependency |
| `[checkout]=trace` | everything inside spans named `checkout` |
| `my_app[checkout{order=7}]=debug` | only where the span's field `order` is 7 |

An event's **target** is its module path (`env_filter::database`), unless it sets its own: `debug!(target: "database", "SELECT 1")`. A program usually reads `RUST_LOG` with a default:

```rust
EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env_lossy()   // a bad directive is skipped with a message, not a crash
```

```bash
RUST_LOG=debug cargo run -p observability                            # bash
$env:RUST_LOG="debug"; cargo run -p observability                    # PowerShell (Remove-Item Env:RUST_LOG after)
```

Real behaviour of the mini project:
- `RUST_LOG=warn` leaves three lines, and they lose their span prefix: **spans have levels too**, and the INFO spans `run_batch` and `job` are filtered out with everything else at INFO.

```text
2026-11-14T14:02:25.962698Z  WARN observability::processor: attempt failed, retrying error=temporary failure: database busy attempt=1
```

- A bad directive is reported and skipped; the rest still applies:

```text
ignoring `observability=nonsense`: error parsing level filter: expected one of "off", "error", "warn", "info", "debug", "trace", or a number 0-5
```

**Testing with `RUST_LOG`:** never change the environment inside a test. One process has one environment, shared by every test thread, so `std::env::set_var` is an `unsafe fn` in edition 2024 (the real error):

```text
error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
 --> setvar.rs:3:5
  |
3 |     std::env::set_var("RUST_LOG", "debug");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
```

Instead, either build the filter from a string in the test (`EnvFilter::new("debug")`, as exercise 5 does), or run the program as a **child process** and set the variable for the child alone, as `tests/cli.rs` does:

```rust
Command::new(env!("CARGO_BIN_EXE_observability"))
    .env("RUST_LOG", "warn,observability::processor=debug")   // only for this child
    .output()
```

> 💡 **Analogy:** `RUST_LOG` is the **notification settings** on the tracking website: "tell me only about problems", "tell me everything about parcels going through Leeds". Changing them doesn't change the scanners. And you don't change the settings on the shared family computer during a test; you give the test its **own** account (a child process).

▶ `cargo run -p observability --example env_filter` (then set `RUST_LOG` yourself for its last section)

## 7. JSON output

For a log collector (Loki, Elasticsearch, CloudWatch, or just `jq`), one JSON object per line is far easier to process than text (the `json` feature):

```text
--- .json(): fields nested, current span AND the list of all spans
{"level":"INFO","fields":{"message":"query finished","rows":3},"target":"json_output","span":{"table":"orders","name":"db_query"},"spans":[{"method":"GET","path":"/orders","name":"request"},{"table":"orders","name":"db_query"}]}
--- .json().flatten_event(true).with_span_list(false)
{"level":"INFO","message":"query finished","rows":3,"target":"json_output","span":{"table":"orders","name":"db_query"}}
```

Fields keep their types (`"rows":3` is a number). The mini project writes JSON with `cargo run -p observability -- --json`, and `tests/cli.rs` parses every line with `serde_json`.

> 💡 **Analogy:** the same scan, printed two ways: a **sentence** for the person at the counter, and a **barcode** for the sorting machine. The machine can't read handwriting, and nobody wants to read barcodes.

▶ `cargo run -p observability --example json_output`

## 8. Async spans with tokio

An async task **pauses** at every `.await` and other tasks run on the same thread meanwhile. A span must follow the **task**, not the thread:

- `#[instrument]` on an `async fn` instruments the future: the span is entered every time the future runs and exited every time it pauses.
- For an async block, use `.instrument(span)` (`use tracing::Instrument`).
- `tokio::spawn` starts a task with **no** span; `.in_current_span()` attaches the current one. The mini project's `run_batch` does that, so every `job` span sits inside `run_batch`.
- **Never hold a span guard across `.await`.** The guard from `span.enter()` is `Send`, so this compiles, and the span stays entered on the thread while the task is paused. Real output, where `innocent()` belongs to no span:

```text
--- span.enter() held across .await (wrong)
 INFO wrong{name="a"}: I belong to no span
 INFO wrong{name="a"}: finished
--- .instrument(span) (right)
 INFO I belong to no span
 INFO right{name="a"}: finished
--- tokio::spawn: a task starts with NO span unless you give it one
 INFO spawned plainly
 INFO batch: spawned with in_current_span
```

Exercise 3 has the same bug with two requests and a bystander task. Its test fails like this:

```text
  left: [" INFO request{request_id=1}:request{request_id=2}: handled", " INFO request{request_id=1}:request{request_id=2}: handled", " INFO request{request_id=1}:request{request_id=2}: not part of any request"]
 right: [" INFO not part of any request", " INFO request{request_id=1}: handled", " INFO request{request_id=2}: handled"]
```

The **owned** guard from `span.entered()` is not `Send`, so with it `tokio::spawn` refuses the future at compile time (checked with a scratch example; this error has no code):

```text
error: future cannot be sent between threads safely
…
4 |     let _guard = tracing::info_span!("request", request_id).entered();
  |         ------ has type `EnteredSpan` which is not `Send`
5 |     tokio::time::sleep(Duration::from_millis(5)).await;
  |                                                  ^^^^^ await occurs here, with `_guard` maybe used later
```

> 💡 **Analogy:** a delivery driver carries several parcels in the van. If the driver hangs the "parcel 1" label **around their own neck** (`enter()`) and then stops for lunch, every parcel they touch at lunch gets scanned as parcel 1. The label must stay **on the parcel** (`instrument`), whoever carries it.

▶ `cargo run -p observability --example async_spans`

## 9. Compatibility with the `log` crate

Many crates still log with the older [`log`](https://docs.rs/log) facade (`log::info!`). tracing-subscriber's default `tracing-log` feature makes `init()` (and `set_default()`) also install a **`LogTracer`**, which turns each `log` record into a tracing event, so both arrive in one place:

```text
 INFO log_compat: a tracing event, with a field host="db.local"
 INFO log_compat::legacy_library: connecting to db.local
DEBUG log_compat::legacy_library: using TLS 1.3
 WARN legacy::pool: connection pool almost full
```

The other direction exists too: `tracing`'s own `log` feature makes tracing events also produce `log` records, for programs that use a `log` logger such as `env_logger`. `log` records have no spans, and their fields arrive as text in the message.

> 💡 **Analogy:** an old depot still writes scans **by hand**. A clerk (`LogTracer`) types every handwritten note into the new system, so head office sees one list. The notes are a bit plainer (no tracking boxes), but nothing is lost.

▶ `cargo run -p observability --example log_compat`

## 10. Metrics and OpenTelemetry (explained, no exporter)

**Logs** say what happened to one thing; **metrics** are numbers over time (jobs per minute, error rate, the 99th-percentile latency); **traces** follow one request across many services. Together they're the "three pillars".

- **Metrics from traces.** `src/metrics.rs` is a layer that counts events per level, counts job outcomes from the `outcome` field, and times every `job` span. The mini project prints its numbers at the end. Real services export such numbers instead: the [`metrics`](https://docs.rs/metrics) crate is a facade like `log` (`counter!("jobs_total").increment(1)`), with exporters for Prometheus and others.
- **OpenTelemetry** (OTel) is the vendor-neutral standard for traces, metrics and logs. The [`tracing-opentelemetry`](https://docs.rs/tracing-opentelemetry) crate is one more **layer**: it turns your spans into OTel spans, and an exporter (usually OTLP) sends them to a collector, then to Jaeger, Grafana Tempo, Honeycomb and so on. Each request gets a **trace id**; services pass it along in the W3C `traceparent` HTTP header, so one request is one trace across every service it touches.
- None of this changes your instrumentation: the same `#[instrument]` and `info!` feed text, JSON, metrics and OTel. That's the point of the facade.

> 💡 **Analogy:** the scans of one parcel are its **story** (logs and spans). The warehouse **board** saying "412 parcels an hour, 3 damaged" is metrics. When a parcel changes from the post office to a courier company, the tracking number goes **with it**, so you can follow it end to end: that's a distributed trace, and OpenTelemetry is the agreement that every company prints tracking numbers the same way.

## 11. Testing log output

When the logs **are** behaviour (an operator relies on the warning before approving a transfer), test them like any other output.

- **Capture in memory.** Give the fmt subscriber a writer that appends to a shared buffer: a type implementing `MakeWriter` (`src/capture.rs`, exercise 4). Switch off colour and timestamps so the text is the same every run.
- **Scope it to the test.** `tracing::subscriber::set_default(subscriber)` lasts until its guard is dropped and only affects the current **thread**, so tests running in parallel don't see each other's logs. `#[tokio::test]` uses a single-threaded runtime, so spawned tasks are covered too.
- **Assert on JSON** for robust tests: parse the line and check only the fields that matter, instead of comparing whole lines.
- **`with_test_writer()`** is for *seeing* logs in tests, not asserting on them: it prints through the test harness, so output appears only for failing tests or with `-- --show-output`.
- **`RUST_LOG`** behaviour: a child process (section 6).

```rust
let logs = CapturedLogs::default();
let subscriber = tracing_subscriber::fmt()
    .with_writer(logs.clone())
    .with_ansi(false)
    .without_time()
    .with_target(false)
    .finish();
let _guard = tracing::subscriber::set_default(subscriber);

let _ = transfer("ama", "kofi", 250_000);

assert_eq!(
    logs.contents(),
    " WARN large transfer held for approval from=\"ama\" to=\"kofi\" pence=250000\n"
);
```

> 💡 **Analogy:** to check the scanner works, you don't stand in the depot watching the screen. You plug a **recorder** into the scanner for one test parcel, scan it, then play the recording back and check every box was filled in.

▶ `cargo test -p observability --example capture_in_tests`

## The mini project: an instrumented job processor

`src/` runs a batch of six jobs concurrently (emails, image resizes, reports), with retries, and is instrumented the way a real service would be:

| File | What it shows |
|---|---|
| [`src/processor.rs`](src/processor.rs) | `#[instrument]` spans (`run_batch` > `job` > `attempt`), levels chosen for the reader, `field::Empty` + `record`, spawned tasks with `in_current_span` |
| [`src/telemetry.rs`](src/telemetry.rs) | the subscriber: registry + text **or** JSON layer (with a per-layer `EnvFilter`) + metrics layer |
| [`src/metrics.rs`](src/metrics.rs) | a custom `Layer`: events per level, outcomes from a field, `job` span timings via extensions |
| [`src/capture.rs`](src/capture.rs) | `CapturedLogs`, the in-memory `MakeWriter` for tests |
| [`src/jobs.rs`](src/jobs.rs) | the jobs and their deterministic failures |
| [`tests/cli.rs`](tests/cli.rs) | runs the real binary as a child process with `RUST_LOG` set for it alone; parses the JSON lines |

```bash
cargo run -p observability                    # text logs on stderr, summary on stdout
cargo run -p observability -- --json          # one JSON object per log line
RUST_LOG=observability=debug cargo run -p observability    # bash; PowerShell: $env:RUST_LOG=…
cargo test -p observability                   # 8 unit + 5 end-to-end + 3 example + 1 doc test
```

```text
… INFO run_batch{jobs=6}: observability::processor: batch started
… ERROR run_batch{jobs=6}:job{job.id=104 job.kind=email attempts=1}: observability::processor: job failed outcome="failed" error=failed permanently: mailbox bounce@example.com does not exist
… WARN run_batch{jobs=6}:job{job.id=103 job.kind=report}: observability::processor: attempt failed, retrying error=temporary failure: database busy attempt=1
… INFO run_batch{jobs=6}:job{job.id=103 job.kind=report attempts=2}: observability::processor: job done outcome="succeeded" summary=report of 12000 rows
… INFO run_batch{jobs=6}: observability::processor: batch finished succeeded=4 failed=2
succeeded 4, failed 2
events by level: {"DEBUG": 7, "ERROR": 2, "INFO": 6, "TRACE": 7, "WARN": 1}
outcomes: {"failed": 2, "succeeded": 4}
6 job spans closed, 57ms of job time in total
```

(Timestamps shortened to `…`. The order of lines changes from run to run: the jobs run concurrently, which is exactly why each line carries its job's span.) Note the metrics: 7 DEBUG and 7 TRACE events were **counted** although none were **printed**, because only the text layer is filtered. The `job` span on the WARN line has no `attempts` yet: an `Empty` field isn't shown until it's recorded.

> 💡 **Analogy:** the processor is a busy depot with six parcels moving at once. Every scan carries the batch number and the parcel's tracking number, problems are flagged at the right urgency, the board counts everything (even scans nobody asked to see), and the scans can go out as sentences or as barcodes.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `info!(order, …)` with your own struct | E0277 the trait bound `Order: Value` is not satisfied | separate fields (`order.id = order.id`), or `?order` / `%order` |
| `#[instrument]` on a function with a non-`Debug` argument | E0277 `Password` doesn't implement `Debug` | `skip(password)`; don't derive `Debug` on secrets |
| `#[instrument]` with a password, token or big buffer argument that *does* implement `Debug` | compiles, and logs it | `skip(...)` / `skip_all` + `fields(...)` |
| `span.enter()` held across `.await` | compiles; other tasks' lines land in the span | `#[instrument]` or `.instrument(span)` |
| `span.entered()` held across `.await`, then `tokio::spawn` | error: future cannot be sent between threads safely (no code) | the same |
| `tokio::spawn(fut)` inside a span | the task's events have no span | `fut.in_current_span()` or `.instrument(span)` |
| Logs never appear | no subscriber was installed | `tracing_subscriber::fmt().init()` at the start of `main` |
| `fmt().init()` called twice (or in a library) | panics: `Unable to install global subscriber: SetGlobalDefaultError("a global default trace dispatcher has already been set")` | install once, in `main`; libraries never do |
| DEBUG lines don't appear | the default level is INFO | `RUST_LOG=debug` or `with_max_level` |
| A typo in `RUST_LOG` | the directive is ignored, with a message | read the `ignoring …` line |
| `std::env::set_var("RUST_LOG", …)` in a test | E0133 call to unsafe function `set_var` | `EnvFilter::new("…")`, or a child process with `Command::env` |
| `span.record()` the same field twice | the text output shows both values | record once, when the value is final |
| `todo!()` / `unwrap()` in a `Layer` callback | a panic in the subscriber can abort the whole process | handle errors, never panic in a layer |
| Asserting on whole text log lines | breaks when the format changes | JSON output + check the fields you need |
| Expecting `with_test_writer()` to let you assert | it only prints | a `MakeWriter` that captures (exercise 4) |

## Practise

**1. Run the examples:**

```bash
cargo run -p observability --example events_and_levels
cargo run -p observability --example fields_and_values
cargo run -p observability --example spans_and_instrument
cargo run -p observability --example env_filter
cargo run -p observability --example json_output
cargo run -p observability --example layers_and_metrics
cargo run -p observability --example async_spans
cargo run -p observability --example log_compat
cargo test -p observability --example capture_in_tests
```

(Set `NO_COLOR=1` to switch off the colours, for example when copying output into a file.)

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_field_value` | Make it compile (E0277 `Value`) with searchable fields | `cargo test -p observability --features exercises --bin ex01_fix_field_value` |
| 2 | `ex02_fix_instrument_secret` | Make it compile (E0277 `Debug`) without logging the password | `cargo test -p observability --features exercises --bin ex02_fix_instrument_secret` |
| 3 | `ex03_fix_span_across_await` | Make it pass: a span guard held across `.await` mislabels lines | `cargo test -p observability --features exercises --bin ex03_fix_span_across_await` |
| 4 | `ex04_capture_logs` | Implement a capturing `MakeWriter`, pass 3 tests | `cargo test -p observability --features exercises --bin ex04_capture_logs` |
| 5 | `ex05_filter_directives` | Write 4 `RUST_LOG` directives, pass 4 tests | `cargo test -p observability --features exercises --bin ex05_filter_directives` |
| 6 | `ex06_counting_layer` | Write a metrics `Layer` (`on_event`, `on_close`), pass 3 tests | `cargo test -p observability --features exercises --bin ex06_counting_layer` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p observability --features solutions --bins`).

**4. The mini project:** run it with different `RUST_LOG` values and with `--json`. Then extend it: add a `queue_wait_ms` field to the `job` span (record how long a job waited before it started), make the metrics layer track the slowest job, and add a test in `src/processor.rs` that checks the new field appears in the captured output.

## Checklist

- [ ] Events and levels: the five macros, no subscriber means no output, the INFO default, `enabled!`
- [ ] Fields: `name = value`, dotted names, `?` and `%`, the `Value` trait, `Empty` + `record`
- [ ] Spans and `#[instrument]`: `enter`, `in_scope`, nesting, `skip`, `fields`, `ret`, `err`, `level`
- [ ] Subscribers and layers: registry, fmt, per-layer vs global filters, writing a `Layer`
- [ ] `EnvFilter` and `RUST_LOG`: directive syntax, targets, span filters, defaults; child processes in tests, never `set_var`
- [ ] JSON output: `json()`, `flatten_event`, span lists
- [ ] Async spans with tokio: `#[instrument]` on async fns, `.instrument`, `in_current_span`, no guards across `.await`
- [ ] `log` compatibility: `LogTracer`, and tracing's `log` feature
- [ ] Metrics and OpenTelemetry: metrics from layers, the `metrics` facade, `tracing-opentelemetry`, trace ids
- [ ] Testing log output: a capturing `MakeWriter`, `set_default`, JSON assertions, `with_test_writer`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `tracing` separates **recording** (events, spans and fields, in any crate) from **deciding** (one subscriber in `main`: filters, formats, destinations).
- Events have levels and typed fields; spans give every event its context, nest, and follow async tasks when attached with `#[instrument]` or `.instrument`.
- A subscriber is a registry plus layers: text or JSON output, `EnvFilter` from `RUST_LOG`, metrics, OpenTelemetry, each seeing every span and event.
- `log` records join in through `LogTracer`; metrics and distributed traces build on the same instrumentation.
- Log output is testable: capture it in memory per thread, assert on JSON fields, and test `RUST_LOG` through child processes.

**In one sentence for a child:** every parcel gets a tracking number and is scanned at every stop, so later you can find out exactly where any parcel went, how long it took and which ones got damaged, without ever following one around.

**Next:** [07 Serialization](../07_serialization/README.md)
