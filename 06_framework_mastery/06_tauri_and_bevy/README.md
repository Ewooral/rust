<!-- hand-written lesson: gen_course must not overwrite -->
# Tauri and Bevy

**Bevy (games):** [bevy.org](https://bevy.org) · [Quick start: setup](https://bevy.org/learn/quick-start/getting-started/setup/) · [Quick start: ECS](https://bevy.org/learn/quick-start/getting-started/ecs/) · [Quick start: resources](https://bevy.org/learn/quick-start/getting-started/resources/) · [docs.rs/bevy](https://docs.rs/bevy) · [error B0001](https://bevy.org/learn/errors/b0001/), [B0002](https://bevy.org/learn/errors/b0002/)
· **Tauri (desktop apps):** [Tauri 2 docs](https://v2.tauri.app/) · [Prerequisites](https://v2.tauri.app/start/prerequisites/) · [Create a project](https://v2.tauri.app/start/create-project/) · [Calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/) · [Calling the frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) · [Capabilities](https://v2.tauri.app/security/capabilities/) · [docs.rs/tauri](https://docs.rs/tauri)

Two big, optional tracks. **Bevy** is a game engine built around an **ECS** (Entity-Component-System). **Tauri** builds desktop apps whose window is a web page and whose logic is Rust, connected by **IPC** (inter-process communication). Both have large dependency trees that take a long time and a lot of disk space to build, so this lesson doesn't depend on either. Instead you'll rebuild the core idea of each in plain Rust, small enough to read in one sitting:

- a tiny **ECS** (entities, component storages, queries, systems, a schedule, resources, commands) and a text-mode simulation on it: a fox chasing rabbits;
- a simulated **Tauri command layer**: JSON messages from a "page", typed commands, managed state, events, and per-window permissions, with a small notes app on top.

At the end of each half there are exact instructions, **clearly marked as not run here**, for creating a real Bevy game and a real Tauri app when you're ready.

> 💡 **The big picture: a sports day and a restaurant.** **Bevy** runs a school sports day. The children are just **bib numbers** (entities). What each child has is written on separate clipboards, one per kind of thing: a "running shoes" clipboard, a "team colour" clipboard (components). **Helpers** each do one job for everyone, again and again: "everyone with running shoes, move forward", "anyone past the line, write their name down" (systems), in the order on the **timetable** (the schedule), and they all look at the **same big clock** (a resource). **Tauri** is a restaurant: the dining room (the web page) and the kitchen (Rust) are separate rooms, and orders pass through a **hatch on written slips** (JSON messages). A **door list** says which tables may order what (capabilities), the kitchen shares one **pantry** (state), and a **bell** tells the dining room when the specials change (events).

## How to run it

```bash
cargo run -p tauri_and_bevy              # both demos
cargo run -p tauri_and_bevy -- ecs       # the fox and the rabbits
cargo run -p tauri_and_bevy -- ipc       # a notes "app" driven by JSON messages
cargo test -p tauri_and_bevy             # 28 unit tests + 2 doc tests
```

## The plan

| # | Section | File | Draws on | Example |
|---|---|---|---|---|
| 1 | Why ECS? | | 03/07 trait objects | |
| 2 | Entities | `src/ecs.rs` | | `ecs1_entities_and_components` |
| 3 | Components and storages | `src/ecs.rs` | 05/04 type erasure, `Any` | `ecs1_entities_and_components` |
| 4 | Queries and borrow conflicts | `src/ecs.rs` | 03/06 `RefCell` | |
| 5 | Systems and the schedule | `src/ecs.rs` | 03/04 closures, traits | `ecs2_systems_and_resources`, `bevy_function_systems` |
| 6 | Resources and commands | `src/ecs.rs` | | `ecs2_systems_and_resources` |
| 7 | A simulation: the fox and the rabbits | `src/critters.rs` | | `ecs3_critters` |
| 8 | Bevy for real (not run) | | | |
| 9 | Tauri's architecture: two worlds and a hatch | | 04/01 threads | |
| 10 | Commands over JSON | `src/ipc.rs` | 05/07 serde | `ipc1_invoke_json` |
| 11 | State and events | `src/ipc.rs`, `src/notes_app.rs` | 04/01 `Mutex`, `Send`/`Sync` | `ipc2_state_and_events`, `ipc4_backend_thread` |
| 12 | Security: capabilities and untrusted input | `src/ipc.rs`, `src/notes_app.rs` | | `ipc3_capabilities` |
| 13 | Tauri for real (not run) | | | |

# Part 1: Bevy's ECS

## 1. Why ECS?

The object-oriented way to write a game is a class per kind of thing: `Player`, `Enemy`, `FlyingEnemy`, `FlyingEnemyThatShoots`... The combinations explode, and the data for one feature (say, every position) is scattered across thousands of objects in memory.

An **ECS** splits things up the other way:

| | What it is | In this lesson |
|---|---|---|
| **Entity** | an id, nothing more | `Entity { index, generation }` |
| **Component** | a piece of plain data an entity can have | `Position`, `Velocity`, `Rabbit`, `Fox` |
| **System** | a function that works on every entity with certain components | `move_rabbits`, `chase`, `catch` |
| **Resource** | a single shared value that isn't attached to an entity | `Tick`, `Score`, `Arena` |

A "flying enemy that shoots" is just an entity with `Enemy`, `Flying` and `Gun` components, and the "flying" system moves everything with `Flying`, whatever else it is. Behaviour comes from *which data you have*, and can change at run time by adding or removing a component.

> 💡 **Analogy: bibs and clipboards.** At sports day you don't make a special kind of child for "fast runner in the red team who also does the long jump". Every child is a bib number. The clipboards say who has running shoes, who's in red, who's doing long jump. The long-jump helper only looks at the long-jump clipboard.

## 2. Entities

```rust
pub struct Entity { index: u32, generation: u32 }
```

An entity is an id. `index` is its slot in every storage. When it's despawned, the slot goes on a free list and can be reused, but with **generation + 1**, so an old `Entity` value kept somewhere can't accidentally refer to the newcomer. Real output from `ecs1_entities_and_components`:

```text
rock was Entity { index: 2, generation: 0 }, pebble is Entity { index: 2, generation: 1 }: same index 2, generations 0 and 1
the old rock id is alive: false; it finds a Name: None
```

Bevy's `Entity` works the same way (an index plus a generation).

> 💡 **Analogy: bib numbers that get reused.** When a child goes home, their bib number 7 can go to a new child, but with a little "2nd" sticker on it. If someone shouts "bib 7, 1st!", nobody answers, instead of the wrong child turning up.

## 3. Components and storages

There's one **storage per component type**, and slot `i` holds entity `i`'s component:

```rust
pub struct Storage<T> { slots: Vec<Option<T>> }

pub struct World {
    storages: HashMap<TypeId, RefCell<Box<dyn AnyStorage>>>,   // one per component type
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
    // … generations, alive flags, the free list
}
```

- **`TypeId` and `dyn Any`** let one map hold a `Storage<Position>`, a `Storage<Velocity>` and so on: the map is keyed by type, and `downcast_ref::<Storage<T>>()` gets the real type back. The `AnyStorage` trait adds what the world needs without knowing `T`: removing a despawned entity's component.
- **`T: 'static`**: `TypeId::of::<T>()` and `Box<dyn Any>` only work for types without short-lived borrows. Leave the bound out (exercise 1):

```text
error[E0310]: the parameter type `T` may not live long enough
  --> 06_framework_mastery\06_tauri_and_bevy\exercises\ex01_fix_component_lifetime.rs:29:20
   |
29 |             .entry(TypeId::of::<T>())
   |                    ^^^^^^^^^^^^^^^
   |                    |
   |                    the parameter type `T` must be valid for the static lifetime...
   |                    ...so that the type `T` will meet its required lifetime bounds
   |
help: consider adding an explicit lifetime bound
   |
26 |     fn push<T: 'static>(&mut self, component: T) {
   |              +++++++++
```

  The world keeps components as long as it likes, so a component can't borrow something that might disappear: it owns its data (`String`, not `&str`). Bevy's `#[derive(Component)]` has the same requirement.
- **Spawning a bundle:** `world.spawn((Position { x: 0, y: 0 }, Velocity { dx: 1, dy: 0 }, Rabbit))` inserts every component in a tuple. `Bundle` is implemented for tuples of 1 to 4 components by a `macro_rules!` ([05/02](../../05_mastery/02_macros_in_depth/README.md)); Bevy does the same for larger tuples.
- **Two kinds of storage.** `Vec<Option<T>>` is fast to iterate when most entities have the component, but wasteful when 3 entities out of 10,000 do. Bevy also has **sparse sets** for those, which exercise 4 builds: the components packed with no gaps, plus a lookup table from entity to position.

> 💡 **Analogy: one clipboard per kind of thing.** The "running shoes" clipboard has a line for every bib number, with a tick or a blank. The office keeps all the clipboards in one drawer, each labelled with what it records (the `TypeId`). A clipboard may only list things that will still exist tomorrow: "the pencil I'm holding right now" isn't allowed (`'static`).

## 4. Queries and borrow conflicts

A **query** visits every entity that has certain components. Bevy writes it as a system parameter, `Query<(&mut Position, &Velocity)>`; here it's a method that takes a closure:

```rust
world.each2_mut::<Position, Velocity>(|entity, position, velocity| {
    position.x += velocity.dx;   // &mut Position, &Velocity
    position.y += velocity.dy;
});
world.each::<Position>(|entity, position| { /* read-only */ });
let rabbits: Vec<Entity> = world.with::<Rabbit>();     // Query<Entity, With<Rabbit>>
```

Every storage is in a `RefCell` ([03/06](../../03_abstraction/06_smart_pointers/README.md)), so a system can write one storage while reading another, through a shared `&World`. Asking for the **same** storage as `&mut` and `&` at once would break Rust's borrowing rules, so `each2_mut::<Position, Position>` refuses with a panic ("conflicting access"). Bevy checks this when it sets the system up and reports **error B0001** (two queries on the same component, one of them mutable).

The `chase` system meets this for real: the fox's `Position` must be written while the rabbits' `Position`s are read, and they're in the same storage. The fix is the usual ECS one: **collect first, then write**:

```rust
let rabbits: Vec<Position> = world.with::<Rabbit>().into_iter()
    .filter_map(|r| world.get::<Position>(r)).collect();          // read everything you need…
for fox in world.with::<Fox>() { /* …then move the fox */ }
```

> 💡 **Analogy: one clipboard, one pen.** Two helpers can both *read* the "positions" clipboard at the same time, but if one is writing on it, nobody else may touch it. So the fox's helper first copies down where all the rabbits are, hands the clipboard back, and only then picks up the pen to move the fox.

## 5. Systems and the schedule

A **system** is a plain function; a **schedule** runs a list of them, in order, once per frame:

```rust
pub type System = fn(&mut World);

pub fn schedule() -> Schedule {
    Schedule::new()
        .add("advance_tick", advance_tick)
        .add("spawn_rabbits", spawn_rabbits)
        .add("move_rabbits", move_rabbits)
        .add("chase", chase)
        .add("catch", catch)
}
```

In Bevy, systems don't take `&mut World`. They **ask for exactly what they need** in their parameters, and Bevy fetches it:

```rust
fn add_points(time: Res<Time>, mut score: ResMut<Score>) { … }
app.add_systems(Update, add_points);
```

How can `add_systems` accept functions with *different* parameter lists? Traits implemented for functions. `examples/bevy_function_systems.rs` rebuilds the trick in about 100 lines: a `SystemParam` trait for things a system can ask for (`Res<T>`, `ResMut<T>`), and an `IntoSystem` trait implemented for every `FnMut(P1)` and `FnMut(P1, P2)` whose parameters are `SystemParam`s (Bevy uses a macro to cover up to 16). Real output:

```text
frame 1:
score is now 5
frame 2:
score is now 15
frame 3:
score is now 30
```

Knowing each system's parameters also lets Bevy see which systems touch **different** data and run those **in parallel** on several threads. Our schedule runs one after another.

> 💡 **Analogy: helpers with job cards.** Each helper has one job card: "move everyone with running shoes". The timetable says the order: whistle, run, check the finish line. In Bevy's version, each job card also lists what the helper needs ("the clock, read only; the score sheet, to write on"), so the organiser can let two helpers work at the same time when they don't need the same things.

▶ `cargo run -p tauri_and_bevy --example bevy_function_systems`

## 6. Resources and commands

**Resources** are one-of-a-kind values: the score, the time, the arena size.

```rust
world.insert_resource(Score::default());                  // app.insert_resource(...)
let tick = world.resource::<Tick>().0;                     // Res<Tick>
world.resource_mut::<Score>().caught += 1;                 // ResMut<Score>
```

A missing resource panics with its type name, as Bevy does. Bevy refuses a system that asks for `Res<T>` and `ResMut<T>` of the same type: [error B0002](https://bevy.org/learn/errors/b0002/).

**Commands** queue changes for later. A system that despawns entities *while* looping over a storage would be changing the clipboard it's reading, so it records the change instead, and applies the queue when the loop is done:

```rust
let mut commands = Commands::default();
world.each::<Position>(|entity, position| {
    if position.x >= finish { commands.despawn(entity); }   // not yet…
});
commands.apply(world);                                       // …now
```

Bevy's `Commands` work the same way, and are applied between systems.

> 💡 **Analogy: the big clock and the notepad.** Everyone looks at the same big clock on the wall (a resource): there's only one. And a helper who notices "bib 12 has finished" while walking along the line writes it on a notepad, and crosses the name off the clipboard *after* finishing the walk, not halfway through (commands).

▶ `cargo run -p tauri_and_bevy --example ecs2_systems_and_resources`

## 7. A simulation: the fox and the rabbits

`src/critters.rs` puts it together: components (`Position`, `Velocity`, `Rabbit`, `Fox`, `Glyph`, `Name`), resources (`Arena`, `Tick`, `Score`, `Log`, and a seeded `Rng` so every run, and every test, is the same), five systems, and a `render` function. Real output of `cargo run -p tauri_and_bevy -- ecs` (first and last frames):

```text
systems, in order: advance_tick -> spawn_rabbits -> move_rabbits -> chase -> catch

tick   0  rabbits 3  caught 0
F.......................
........................
......................r.
........................
..............r.........
..............r.........
........................
........................
…
tick  30  rabbits 2  caught 8
........................
........................
........................
.............r..........
........................
..F.....................
......................r.
........................

what happened:
  tick 4: a rabbit appears at (18, 2)
  tick 7: Rusty caught a rabbit at (7, 2)
  …
```

Adding a feature in an ECS means adding **data and systems**, without touching the existing ones. Exercise 3 makes foxes hungry: a `Hunger` component and three new systems, scheduled after the game's own. The fox then gives up during a long dry spell, one tick before it would have caught another rabbit:

```text
  tick 20: Rusty caught a rabbit at (11, 4)
  tick 24: a rabbit appears at (4, 0)
  tick 28: a rabbit appears at (21, 6)
  tick 28: Rusty went home hungry
```

> 💡 **Analogy: adding a new event to sports day.** To add an egg-and-spoon race, you don't rebuild the children. You hand out spoons (a new component) and add a helper with a new job card (a new system) to the timetable.

▶ `cargo run -p tauri_and_bevy --example ecs3_critters -- --watch` (animated in a terminal; `--seed 7` for a different field)

## 8. Bevy for real (not run)

> **Not run in this course.** These are the steps from Bevy's own setup guide. A first Bevy build compiles a large dependency tree: allow plenty of time and disk space.

**1. System requirements.**
- **Windows:** run the [Visual Studio C++ Build Tools installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the **Desktop development with C++** workload (or at least the latest MSVC and Windows SDK).
- **Linux (Ubuntu):** from Bevy's [Linux dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md) page:

```bash
sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0
sudo apt-get install libwayland-dev libxkbcommon-dev      # Wayland support
```

- Bevy's minimum supported Rust version is the latest stable release, so `rustup update` first.

**2. A project** (outside this workspace):

```bash
cargo new my_bevy_game
cd my_bevy_game
cargo add bevy
```

**3. Faster development builds**, in that project's `Cargo.toml`:

```toml
[profile.dev]
opt-level = 1            # a little optimisation for your code

[profile.dev.package."*"]
opt-level = 3            # full optimisation for dependencies (built once)
```

and `cargo run --features bevy/dynamic_linking` while developing (it links Bevy as a shared library, so rebuilds are quicker).

**4. The same ideas, with the real engine:**

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { x: f32, y: f32 }

fn spawn(mut commands: Commands) {
    commands.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.5 }));
}

fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)          // window, rendering, input, time, assets…
        .add_systems(Startup, spawn)          // runs once
        .add_systems(Update, movement)        // runs every frame
        .run();
}
```

| This lesson | Bevy |
|---|---|
| `World::spawn((A, B))` | `commands.spawn((A, B))` |
| any `'static` type | `#[derive(Component)]`, `#[derive(Resource)]` |
| `world.each2_mut::<A, B>(…)` | `Query<(&mut A, &B)>` |
| `world.with::<Rabbit>()` | `Query<Entity, With<Rabbit>>` |
| `world.resource::<T>()` / `resource_mut` | `Res<T>` / `ResMut<T>` |
| `Schedule::new().add(…)` | `app.add_systems(Update, …)` |
| systems run in order | systems run in parallel when their data doesn't overlap |

**What to learn next:** Bevy's [quick start](https://bevy.org/learn/quick-start/getting-started/setup/) (ECS, resources, plugins), then the official examples in the [Bevy repository](https://github.com/bevyengine/bevy/tree/main/examples) (2D, input, UI, audio, states, events). Read [docs.rs/bevy](https://docs.rs/bevy) for `Query`, `Commands`, `Time`, and `Transform`.

> 💡 **Analogy: from a cardboard model to the real stadium.** You've built sports day with cardboard and string, so you know where every helper stands and why. Bevy is the real stadium, with floodlights, a scoreboard and a sound system (rendering, input, audio) and a lot more staff, but the timetable and the clipboards work the same way.

# Part 2: Tauri's command layer

## 9. Tauri's architecture: two worlds and a hatch

A Tauri app has two halves:

- the **frontend**: an ordinary web page (HTML, CSS, JavaScript or a framework), shown by the operating system's **webview** (Microsoft Edge WebView2 on Windows, WebKitGTK on Linux), instead of shipping a whole browser;
- the **core**: your Rust code, with full access to files, the network and the operating system.

They can't call each other's functions. The page calls `invoke("add_note", { title: "milk" })`; Tauri serialises that to a message, passes it to the Rust side over **IPC**, runs your command, and sends the serialised result back to resolve the JavaScript promise. Rust can also push **events** to the page. This lesson's `src/ipc.rs` does each step with `serde_json` strings so you can watch it:

```text
page -> rust: {"id":2,"window":"main","cmd":"add_note","args":{"title":"Buy milk"}}
rust -> page: {"id":2,"ok":{"id":1,"isDone":false,"title":"Buy milk"}}

page -> rust: {"id":3,"window":"main","cmd":"add_note","args":{"title":""}}
rust -> page: {"id":3,"error":"a note needs a title"}
```

`id` pairs each reply with the right promise: replies can arrive in any order.

> 💡 **Analogy: the kitchen hatch.** Diners never walk into the kitchen, and cooks never come out to the tables. Orders go through the hatch on a slip with a table number and a ticket number; the dish (or "sorry, we're out of that") comes back with the same ticket number, so it reaches the right table.

## 10. Commands over JSON

In a real Tauri app a command is a function with an attribute:

```rust
#[tauri::command]
fn add_note(title: String, notes: tauri::State<'_, Mutex<Notes>>) -> Result<Note, String> { … }
```

What `#[tauri::command]` generates is roughly what `Router::command` does by hand: **deserialize the arguments, call the function, serialize the result**:

```rust
pub fn command<A, R, F>(mut self, name: &'static str, handler: F) -> Self
where
    A: DeserializeOwned,
    R: Serialize,
    F: Fn(&Context<'_>, A) -> Result<R, String> + Send + Sync + 'static,
{
    let erased: Handler = Box::new(move |ctx, args| {
        let args: A = serde_json::from_value(args)
            .map_err(|e| format!("invalid arguments for {name}: {e}"))?;
        let result = handler(ctx, args)?;
        serde_json::to_value(result).map_err(|e| format!("cannot send the result: {e}"))
    });
    self.handlers.insert(name, erased);
    self
}
```

Every command, whatever its argument and result types, becomes the same boxed type, `Fn(&Context, Value) -> Result<Value, String>` ("type erasure"), so they fit in one `HashMap`.

- **Names:** JavaScript uses camelCase (`{ noteId: 1 }`) and Rust snake_case (`note_id`). Tauri converts command arguments for you; here the argument structs say `#[serde(rename_all = "camelCase")]`.
- **Errors** become a rejected promise. In Tauri, everything a command returns must implement `serde::Serialize`, **including the error type**.
- **Key order:** the replies show `{"id":1,"isDone":false,"title":"Buy milk"}` with the keys sorted. JSON objects have no order, and `serde_json::Value` stores objects sorted by key unless its `preserve_order` feature is enabled.

Real output from `ipc1_invoke_json`, showing what the page gets back for bad input:

```text
negative size     -> {"id":2,"error":"sizes can't be negative"}
missing argument  -> {"id":3,"error":"invalid arguments for area: missing field `heightCm`"}
wrong type        -> {"id":4,"error":"invalid arguments for area: invalid type: string \"three\", expected f64"}
snake_case keys   -> {"id":5,"error":"invalid arguments for area: missing field `widthCm`"}
```

> 💡 **Analogy: the order slip.** The kitchen only accepts slips filled in on the right form: every box named exactly as printed, numbers in the number boxes. A slip that says "three" in the number box comes straight back with a note saying which box is wrong, before any cook touches a pan.

▶ `cargo run -p tauri_and_bevy --example ipc1_invoke_json`

## 11. State and events

**State** is data every command shares: Tauri's `Builder::manage(value)`, read in commands as a `tauri::State<'_, T>` parameter. Here, `Router::manage` stores it by type and commands get it from their `Context`:

```rust
let backend = Router::new()
    .manage(Mutex::new(Notes::default()))   // .manage(...) in tauri::Builder
    .command("add_note", add_note);          // generate_handler![add_note]

pub fn add_note(ctx: &Context<'_>, args: AddNoteArgs) -> Result<Note, String> {
    let mut notes = ctx.state.get::<Mutex<Notes>>()?.lock().map_err(|_| "unavailable")?;
    …
    ctx.events.emit("notes-changed", notes.len());    // app.emit("notes-changed", …)
    Ok(note)
}
```

- Commands can run on **several threads at once**, so state must be `Send + Sync`, and data that changes goes in a `Mutex` ([04/01](../../04_systems_level/01_fearless_concurrency/README.md)). The handlers themselves must be `Send + Sync` too. Exercise 2 moves a command table to a backend thread and meets both errors:

```text
error[E0277]: `dyn for<'a> Fn(&'a str) -> String` cannot be sent between threads safely
   --> 06_framework_mastery\06_tauri_and_bevy\exercises\ex02_fix_handler_send.rs:42:33
    |
 42 |       let backend = thread::spawn(move || {
    |  ___________________-------------_^
…
    = help: the trait `Send` is not implemented for `dyn for<'a> Fn(&'a str) -> String`
```

  and, once the alias says `+ Send + Sync`, `` error[E0277]: `Rc<Cell<u32>>` cannot be sent between threads safely `` for the handler that captured an `Rc`.
- **Events** go the other way, unasked: Rust calls `app.emit("notes-changed", payload)` (the `tauri::Emitter` trait), and the page listens with `listen("notes-changed", handler)` from `@tauri-apps/api/event`. The payload must be serialisable and `Clone`. Here, `Events::emit` queues them and `take_events` hands them to the "page".
- `ipc4_backend_thread` runs the router on its own thread and talks to it through channels, which is the shape of the real thing: two sides exchanging messages, never sharing a call stack.

> 💡 **Analogy: the pantry and the bell.** All the cooks share one pantry (state), and a lock on the pantry door makes sure two cooks don't grab the last egg at once (`Mutex`). When the specials board changes, the kitchen rings a bell that every dining room can hear (an event), so nobody has to keep walking to the hatch to ask.

▶ `cargo run -p tauri_and_bevy --example ipc2_state_and_events`, then `--example ipc4_backend_thread`

## 12. Security: capabilities and untrusted input

The page is a web page. A bug, a malicious dependency in its JavaScript, or injected content could make it send **any** message. So the Rust side decides what's allowed, and checks everything it receives.

**Capabilities.** In Tauri 2, capability files in `src-tauri/capabilities/` (JSON or TOML) say which permissions each window gets. (Tauri 1 had an "allowlist" in its config instead; the migration guide says it was rewritten into this permission system.) This lesson's version, deny by default:

```json
[
  { "identifier": "main-window",      "windows": ["main"],
    "commands": ["greet", "add_note", "list_notes", "toggle_note", "delete_note"] },
  { "identifier": "widget-read-only", "windows": ["widget"],
    "commands": ["greet", "list_notes"] }
]
```

Real output from `ipc3_capabilities`:

```text
widget             list_notes         ok: [{"id":1,"isDone":false,"title":"Secret plans"}]
widget             delete_note        REFUSED: command delete_note not allowed for window widget
devtools-injected  list_notes         REFUSED: command list_notes not allowed for window devtools-injected
main               delete_everything  REFUSED: command delete_everything not allowed for window main
main               add_note           REFUSED: invalid arguments for add_note: unknown field `isAdmin`, expected `title`
```

One difference to know: in real Tauri 2, the **core and plugin APIs** need permissions granted in a capability, but your **own commands are allowed for every window by default**; the capabilities page explains how to restrict them (by declaring them in `build.rs` with `AppManifest::commands`). The simulation denies everything by default, which is the safer habit to learn.

**Untrusted input**, checked in the commands themselves:
- `#[serde(deny_unknown_fields)]` on argument structs: `{"title": "x", "isAdmin": true}` is refused instead of silently half-accepted.
- Validate values: `check_title` trims, rejects empty titles and limits the length.
- Check permission **before** looking the command up, so a window that isn't allowed can't even learn which commands exist.
- File paths from the page must stay inside an allowed folder. Exercise 6 writes the check with `Path::components()`, refusing `..` that climbs out, absolute paths and Windows drive letters. (Tauri's file-system plugin has "scopes" for this.)

> 💡 **Analogy: the door list and the careful cook.** The doorman has a list: table "main" may order anything, the little "widget" table only drinks, and anyone not on the list gets nothing. And even an order from an allowed table is read carefully: a slip that adds "and make me the manager" in the margin is sent back.

▶ `cargo run -p tauri_and_bevy --example ipc3_capabilities`

## 13. Tauri for real (not run)

> **Not run in this course.** These are the steps from Tauri 2's own guides.

**1. Prerequisites.**
- **Windows:** install the **Microsoft C++ Build Tools** with **Desktop development with C++** checked. Tauri uses **Microsoft Edge WebView2**, which the guide says is already installed on Windows 10 (from version 1803) and later versions of Windows. Rust must use the **MSVC** toolchain: if you already have Rust, `rustup default stable-msvc`.
- **Linux (Debian/Ubuntu):**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**2. Create and run a project** (outside this workspace):

```bash
cargo install create-tauri-app --locked
cargo create-tauri-app                    # asks: project name, identifier, frontend language
                                          # (Rust, TypeScript/JavaScript or .NET), package manager,
                                          # UI template (e.g. Vanilla, React, Svelte, Leptos, Yew)
cd tauri-app
cargo install tauri-cli --version "^2.0.0" --locked
cargo tauri dev                           # with an npm template: npm install, then npm run tauri dev
```

**3. The same ideas, with the real framework.** `src-tauri/src/lib.rs`:

```rust
use std::sync::Mutex;
use tauri::{Emitter, State};

#[derive(Default)]
struct Notes(Vec<String>);

#[tauri::command]
fn add_note(title: String, notes: State<'_, Mutex<Notes>>, app: tauri::AppHandle) -> Result<usize, String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("a note needs a title".into());       // rejects the JS promise
    }
    let mut notes = notes.lock().map_err(|_| "notes unavailable".to_string())?;
    notes.0.push(title);
    app.emit("notes-changed", notes.0.len()).map_err(|e| e.to_string())?;
    Ok(notes.0.len())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Notes::default()))
        .invoke_handler(tauri::generate_handler![add_note])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

and in the page:

```js
import { invoke } from '@tauri-apps/api/core';     // Tauri 1 used '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event';

await listen('notes-changed', (event) => console.log('notes:', event.payload));
const count = await invoke('add_note', { title: 'Buy milk' });
```

| This lesson | Tauri 2 |
|---|---|
| `Router::command("add_note", add_note)` | `#[tauri::command]` + `generate_handler![add_note]` |
| `Frontend::invoke(…)` | `invoke('add_note', { … })` from `@tauri-apps/api/core` |
| `Router::manage(Mutex::new(…))`, `ctx.state.get::<T>()` | `Builder::manage(…)`, a `State<'_, T>` parameter |
| `ctx.events.emit(…)`, `take_events()` | `app.emit(…)` (`Emitter`), `listen(…)` in the page |
| capability JSON, deny by default | `src-tauri/capabilities/*.json`; own commands allowed unless restricted |
| `#[serde(rename_all = "camelCase")]` on argument structs | automatic for command parameters |

When it works, `cargo tauri build` makes the release build and installers; the Tauri docs' Distribute section covers signing and platform packages.

**What to learn next:** [Calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/) (async commands, errors, channels), [Calling the frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) (events), [Capabilities](https://v2.tauri.app/security/capabilities/) and the rest of the Security section, then the official plugins (file system, dialog, store, HTTP).

> 💡 **Analogy: from a pretend restaurant to a real one.** You've played restaurant with paper slips and a cardboard hatch, so you know why the slips have ticket numbers and why the doorman has a list. Tauri is a real restaurant with a real building (the webview) and a real hatch (IPC): opening one is mostly following the recipe above.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| A generic storage without `T: 'static` | E0310 the parameter type `T` may not live long enough | `T: 'static` (Bevy: `#[derive(Component)]`) |
| A component holding `&str` from a local `String` | refused: not `'static` | Own the data: `String`. |
| `&mut Position` and `&Position` in one query | panic here; Bevy error B0001 | Collect what you read first, then write. |
| `Res<T>` and `ResMut<T>` of one type in one system | panic here; Bevy error B0002 | Ask once, as `ResMut<T>`. |
| Despawning inside a loop over a storage | a borrow conflict | Queue it in `Commands`, apply after the loop. |
| Keeping an `Entity` after despawning it | (with generations) it safely finds nothing | Check `is_alive`, or don't keep stale ids. |
| `Box<dyn Fn(..)>` handlers moved to another thread | E0277 cannot be sent between threads safely | `+ Send + Sync` on the trait object |
| `Rc`/`Cell`/`RefCell` captured by a command or in state | E0277 not `Send`/`Sync` | `Arc`, `Mutex`, atomics |
| A command's error type without `Serialize` (Tauri) | it won't compile as a command | Return `Result<T, String>` or a serialisable error. |
| `invoke('toggle_note', { note_id: 1 })` | the argument is missing (here: missing field `noteId`) | camelCase keys in JavaScript. |
| Argument structs without `deny_unknown_fields` | extra keys like `isAdmin` are silently ignored | Add `#[serde(deny_unknown_fields)]`. |
| Registered a command but didn't grant it | "not allowed for window main" (here) | Add it to the capability file. |
| `path.contains("..")` as a security check | misses tricks, rejects `notes..txt` | Check `Path::components()`. |
| `import { invoke } from '@tauri-apps/api/tauri'` in Tauri 2 | module not found | `@tauri-apps/api/core` |
| Adding `bevy` or `tauri` to this workspace | very long builds, GBs of disk | Make a separate project (sections 8 and 13). |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p tauri_and_bevy --example ecs1_entities_and_components
cargo run -p tauri_and_bevy --example ecs2_systems_and_resources
cargo run -p tauri_and_bevy --example ecs3_critters
cargo run -p tauri_and_bevy --example bevy_function_systems
cargo run -p tauri_and_bevy --example ipc1_invoke_json
cargo run -p tauri_and_bevy --example ipc2_state_and_events
cargo run -p tauri_and_bevy --example ipc3_capabilities
cargo run -p tauri_and_bevy --example ipc4_backend_thread
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_component_lifetime` | Make it compile (E0310): `'static` components | `cargo run -p tauri_and_bevy --features exercises --bin ex01_fix_component_lifetime` |
| 2 | `ex02_fix_handler_send` | Make it compile (E0277, twice): `Send + Sync` handlers, `Arc` not `Rc` | `cargo run -p tauri_and_bevy --features exercises --bin ex02_fix_handler_send` |
| 3 | `ex03_hungry_foxes` | Extend the game: a component and three systems, pass 4 tests | `cargo test -p tauri_and_bevy --features exercises --bin ex03_hungry_foxes` |
| 4 | `ex04_sparse_set` | Bevy's other storage: a sparse set, pass 4 tests | `cargo test -p tauri_and_bevy --features exercises --bin ex04_sparse_set` |
| 5 | `ex05_rename_note` | Add a command: write, register and allow it, pass 4 tests | `cargo test -p tauri_and_bevy --features exercises --bin ex05_rename_note` |
| 6 | `ex06_path_scope` | Keep page-supplied paths inside a folder, pass 4 tests (5 on Windows) | `cargo test -p tauri_and_bevy --features exercises --bin ex06_path_scope` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p tauri_and_bevy --features solutions --bins`, and `cargo run -p tauri_and_bevy --features solutions --bin sol03_hungry_foxes`, etc.).

**4. Projects:**
- **ECS:** add a second fox with its own name; give rabbits a `Fear` component so they run away from a fox within 3 squares; or build a different game on `ecs.rs`: snake, or falling sand.
- **IPC:** add a `search_notes { query }` command with a capability for the widget window; make `toggle_note` emit a `note-toggled` event carrying the note; write a "page" in a loop that reads commands typed on stdin and prints the JSON replies.
- **For real:** when you have the disk space, follow section 8 or 13 in a separate folder and port the fox or the notes app.

## Checklist

- [ ] Entities as ids with generations; components as plain data stored by type
- [ ] `TypeId`, `Box<dyn Any>` and downcasting for type-keyed storage, and why components are `'static`
- [ ] Queries, and why `&mut A` with `&A` conflicts (Bevy B0001)
- [ ] Systems as functions, a schedule, and how Bevy turns parameters into data access
- [ ] Resources (Bevy B0002) and deferred `Commands`
- [ ] Bevy's real setup: build tools, Linux packages, `cargo add bevy`, dev-profile optimisation, `dynamic_linking`
- [ ] Tauri's architecture: webview frontend, Rust core, IPC messages with ids
- [ ] Commands as deserialize → call → serialize, with type erasure; camelCase arguments; serialisable errors
- [ ] Managed state that is `Send + Sync`, and events from Rust to the page
- [ ] Capabilities per window (and Tauri 2's default for your own commands), strict argument types, path scopes
- [ ] Tauri's real setup: prerequisites (WebView2, MSVC, Linux packages), `create-tauri-app`, `cargo tauri dev`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- An **ECS** keeps data (components, stored per type) apart from behaviour (systems, run by a schedule); entities are just ids, resources are shared singletons, and commands defer structural changes.
- Rust's borrowing rules shape an ECS: one writer or many readers per storage, `'static` components, and conflicts that Bevy reports as B0001 and B0002.
- **Tauri** splits an app into a web page and a Rust core that exchange serialised messages: commands (request and reply), and events (Rust to page).
- A command is "deserialize, call, serialize"; shared state must be thread-safe; and everything from the page is untrusted, so check permissions, argument shapes, values and paths.
- Both frameworks are big to build: learn the ideas here, then follow the not-run setup steps in a separate project.

**In one sentence for a child:** Bevy runs a game like a sports day, with numbered children, clipboards and helpers who each do one job, and Tauri runs an app like a restaurant, where the dining room and the kitchen only talk through a hatch on written slips, and the doorman checks who may order what.

**Next:** [07 Capstone: a REST API](../07_capstone_rest_api/README.md)
