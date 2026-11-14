<!-- hand-written lesson: gen_course must not overwrite -->
# Serialization with Serde

**Serde:** [serde.rs](https://serde.rs/) (the guide) · [Data model](https://serde.rs/data-model.html) · [Container](https://serde.rs/container-attrs.html), [variant](https://serde.rs/variant-attrs.html) and [field attributes](https://serde.rs/field-attrs.html) · [Enum representations](https://serde.rs/enum-representations.html) · [Custom deserialization](https://serde.rs/deserialize-struct.html) · [Borrowing (zero-copy)](https://serde.rs/lifetimes.html)
· API docs: [`serde`](https://docs.rs/serde), [`serde_json`](https://docs.rs/serde_json), [`toml`](https://docs.rs/toml), [`postcard`](https://docs.rs/postcard)

Run it: `cargo run -p serialization`  ·  Test it: `cargo test -p serialization`

Almost every program reads or writes data: config files, web API requests, save games, messages between services. **Serialization** turns a Rust value into bytes or text, and **deserialization** turns them back. In Rust nearly everyone uses one framework for this, [Serde](https://serde.rs/), and this lesson is also the course's Serde framework lesson. After it you'll be able to:

- derive `Serialize` and `Deserialize`, and read and write JSON, TOML and a compact binary format with the same types;
- shape the output with attributes (`rename_all`, `default`, `flatten`, `deny_unknown_fields`, ...) and choose how enums look;
- write custom (de)serializers for one field, and a full `Deserialize` impl with a hand-written `Visitor`;
- deserialize without copying (`&'de str`, `Cow`), and know when you need `DeserializeOwned`;
- handle data of unknown shape with `serde_json::Value`, give users errors with a line and column, and **migrate old file formats** to new ones.

> 💡 **The big picture: posting a toy castle.** You want to send your Lego castle to a friend. You can't post a castle, so you **take it apart** into pieces and write an **instruction leaflet** (serialize). Your friend **follows the leaflet** to build it again (deserialize). **Serde** is the leaflet *language* that every toy and every box understands. The **formats** are different boxes: a see-through box with labels anyone can read (JSON), a tidy drawer with named compartments for grown-ups to edit (TOML), and a vacuum-packed bag that's tiny but only makes sense if you know exactly what's inside (postcard). **Attributes** are notes on the leaflet ("call this piece *type*"), and **migration** is reading a leaflet written years ago for an older castle.

## 1. How Serde works: two halves and a shared language

Serde splits the job in two, so that nothing has to know about everything:

| Your types implement | Formats implement | Meeting point |
|---|---|---|
| `Serialize`: "describe yourself" | `Serializer`: "write what I'm told" | the [Serde data model](https://serde.rs/data-model.html): 29 types such as `bool`, `u8`…`u128`, `f64`, `char`, `str`, `bytes`, `option`, `unit`, `seq`, `tuple`, `map`, `struct`, enum variants |
| `Deserialize`: "here's a `Visitor`, call it with what you find" | `Deserializer`: "read the input, call the Visitor" | the same 29 types |

So a struct describes itself as "a struct with these fields", and `serde_json` decides that means `{"name": ...}` while `toml` writes `name = ...`. **M types and N formats need M + N pieces of code, not M × N.**

> 💡 **Analogy: pen pals with one shared alphabet.** Children in a hundred countries want to write to each other. Instead of every child learning every language, everyone learns **one shared alphabet** (the data model). Each child only needs to know how to translate their own words into it (your types), and each post office only needs to know how to print it on its own paper (the formats).

## 2. Derive, and JSON with `serde_json`

```rust
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Pet {
    name: String,
    species: Species,          // an enum: derived below it in the file
    age: u8,
    toys: Vec<String>,
    microchip: Option<u64>,
}

let compact = serde_json::to_string(&rex)?;          // Rust -> JSON text
let pretty = serde_json::to_string_pretty(&rex)?;
let back: Pet = serde_json::from_str(&compact)?;      // JSON text -> Rust
```

```text
{"name":"Rex","species":"Dog","age":3,"toys":["ball","rope"],"microchip":null}
```

- The derive writes `impl Serialize for Pet` and `impl Deserialize for Pet`. **Every field's type must implement the trait too**, so nested structs need the derive as well (exercise 1).
- `from_str` needs to know *what* to build: the `: Pet` annotation (or `from_str::<Pet>`) chooses it.
- Field order in the input doesn't matter. `None` is written as `null`, a unit enum variant as a string (`"Dog"`), a map as an object, a tuple as an array.
- `to_writer`/`from_reader` work with any `io::Write`/`io::Read` (files, sockets), and `from_slice` reads bytes you already have.

Forget the derive on a nested type, and the error points at the outer derive and the field (exercise 1):

```text
error[E0277]: the trait bound `Lesson: serde::Serialize` is not satisfied
    --> 05_mastery\07_serialization\exercises\ex01_fix_missing_derive.rs:16:28
     |
  16 | #[derive(Debug, PartialEq, Serialize, Deserialize)]
     |                            ^^^^^^^^^ unsatisfied trait bound
...
  19 |     lessons: Vec<Lesson>,
     |     ------- required by a bound introduced by this call
     |
help: the trait `Serialize` is not implemented for `Lesson`
    --> 05_mastery\07_serialization\exercises\ex01_fix_missing_derive.rs:23:1
     |
  23 | struct Lesson {
     | ^^^^^^^^^^^^^
     = note: for local types consider adding `#[derive(serde::Serialize)]` to your `Lesson` type
     = note: for types from other crates check whether the crate offers a `serde` feature flag
...
     = note: required for `Vec<Lesson>` to implement `Serialize`
```

The second note matters for crates you don't own: many (`chrono`, `uuid`, `url`...) implement Serde's traits behind a `serde` feature flag, so you switch it on in `Cargo.toml` instead of writing anything.

> 💡 **Analogy: every brick needs its own instructions.** The leaflet for the castle says "now build the tower". If the tower has no instructions of its own, nobody can finish the castle, and the helper tells you exactly which piece is missing its leaflet.

▶ `cargo run -p serialization --example derive_and_json`

## 3. One type, many formats

The same `Recipe` struct, written three ways (from the example):

| Format | Crate | Size | Good for |
|---|---|---|---|
| JSON | `serde_json` | 135 bytes | web APIs, anything that talks to JavaScript |
| TOML | `toml` | 148 bytes | config files people edit (`Cargo.toml`!) |
| postcard | `postcard` | 29 bytes | messages between programs, embedded devices (lesson 08) |

```text
postcard (29 bytes):
[8, 80, 97, 110, 99, 97, 107, 101, 115, 4, 172, 2, 1, 2, 5, 102, 108, 111, 117, 114, 200, 1, 4, 109, 105, 108, 107, 172, 2]
```

That's the string length 8 and the letters of "Pancakes", then `4` (serves), `172, 2` (300 minutes, as a 2-byte *varint*), `1` (true), `2` (two ingredients)... There are **no field names**: postcard trusts the reader to use the same struct.

Each format has limits, and they're errors, not surprises:

```text
TOML, a bare list:   Err(UnsupportedType(Some("array")))
TOML, a null:        Err(UnsupportedType(Some("unit")))
postcard -> Value:   Err(WontImplement)
postcard, cut short: Err(DeserializeUnexpectedEnd)
postcard, wrong struct: Ok(Imposter { title: "Pancakes", level: 4 })  <- no error!
```

- TOML's top level must be a table (a struct or map), and TOML has no `null`: an `Option` field that is `None` is simply left out.
- JSON and TOML are **self-describing**: the text says "this is a string". postcard isn't, so it can't feed a type that asks "what have you got?" (`serde_json::Value`, or a `Visitor` that uses `deserialize_any`), and reading with the wrong struct can quietly succeed.

This lesson's `Cargo.toml` (the versions Cargo locked here: serde 1.0.229, serde_json 1.0.154, toml 1.2.1, postcard 1.1.3):

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }   # the traits + derive macros
serde_json = "1"
toml = "1"

[dev-dependencies]   # only the examples use it
postcard = { version = "1", default-features = false, features = ["use-std"] }
```

> 💡 **Analogy: three boxes for the same castle.** A see-through box with a label on every bag of pieces (JSON). A drawer with named compartments, easy for a grown-up to tidy (TOML). A vacuum-packed bag with no labels at all: tiny, but your friend must already know the order the pieces go in, or they'll build something odd without noticing (postcard).

▶ `cargo run -p serialization --example formats`

## 4. Field and container attributes

Attributes change the *shape in the file* without changing your Rust code.

| Attribute | On | Does |
|---|---|---|
| `rename_all = "camelCase"` | struct/enum | renames every field or variant (`lowercase`, `UPPERCASE`, `PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`) |
| `rename = "type"` | field/variant | one name, even a Rust keyword |
| `alias = "mail"` | field/variant | *also* accept this name when reading; writing uses the main name |
| `default` / `default = "path"` | field or struct | a missing field gets `Default::default()` or `path()` |
| `skip` | field | never written or read; filled with `Default` |
| `skip_serializing_if = "Vec::is_empty"` | field | leave it out when the function says so |
| `flatten` | field | the inner struct's fields sit at this level; a flattened map catches every unclaimed field |
| `deny_unknown_fields` | struct | an unknown field is an error instead of being ignored |

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    user_id: u64,                                   // "userId"
    #[serde(rename = "type")]
    kind: String,
    #[serde(alias = "e-mail", alias = "mail")]
    email: String,
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
    #[serde(flatten)]
    address: Address,                               // "city", "postCode" at this level
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,     // everything nobody claimed
}
```

Reading `{"userId": 42, ..., "mail": "...", "city": "Accra", "postCode": "GA-123", "plan": "family"}` and writing it back gives `"email"` (not the alias), no `nickname`, the address flattened, and `"plan"` kept from `extra`.

By default Serde **ignores unknown fields**, which is kind to newer clients but hides typos. Compare:

```text
deny_unknown_fields: unknown field `color`, expected `colour` at line 1 column 9
container default, typo ignored: Settings { volume: 7, muted: false }  <- volume is still 7!
```

Two rules of thumb: `deny_unknown_fields` for **config files** (a typo should be loud), the default for **network messages** (an old client should survive a newer server's extra fields). The Serde docs say `deny_unknown_fields` is not supported together with `flatten`.

> 💡 **Analogy: sticky notes on the leaflet.** "We call this piece *type*" (rename). "Some people call it *mail*, that's fine too" (alias). "If this piece is missing, use a spare from the box" (default). "Don't bother packing the empty bag" (skip_serializing_if). "Tip the small bag's pieces straight into the big box" (flatten). And "if you find a piece you don't recognise, STOP and tell me" (deny_unknown_fields).

▶ `cargo run -p serialization --example field_attributes`

## 5. Enum representations

The same enum, four ways (the example's real output):

```text
               Circle                                     Dot
external:      {"Circle":{"radius":1.5}}                  "Dot"
internal:      {"shape":"Circle","radius":1.5}            {"shape":"Dot"}
adjacent:      {"shape":"Circle","data":{"radius":1.5}}   {"shape":"Dot"}
untagged:      {"radius":1.5}                             null
```

| Representation | Attribute | Notes |
|---|---|---|
| Externally tagged | (default) | works for every kind of variant and every format |
| Internally tagged | `#[serde(tag = "shape")]` | the usual web API style (`"type": "..."`); no tuple variants; the tag must be a **string** |
| Adjacently tagged | `#[serde(tag = "shape", content = "data")]` | tag and data side by side |
| Untagged | `#[serde(untagged)]` | no name at all: tries each variant **in order**, keeps the first that fits |

- `#[serde(other)]` on a unit variant of an internally or adjacently tagged enum catches every unknown tag: `{"shape": "Hexagon"}` becomes `Unknown` instead of an error.
- Internally tagged tuple variants are refused by the derive macro itself (no error code, the compiler isn't involved): `error: #[serde(tag = "...")] cannot be used with tuple variants`.
- Untagged is great for "a number **or** a string" (`IdOrName`), but its errors are vague and lose their position: `data did not match any variant of untagged enum Untagged` at line 0, column 0. Internally tagged enums lose it too: to find the tag, Serde buffers the whole object first.

> 💡 **Analogy: labelling a parcel.** You can write the name on the **outside of the box** (external), on a **card inside** with the toy (internal), on a **luggage tag tied next to** the box (adjacent), or **not at all**, so whoever opens it has to guess from the shape (untagged). Guessing works for a ball and a book, but it's slow, and if they guess wrong nobody can say why.

▶ `cargo run -p serialization --example enum_representations`

## 6. Customising one field: `with`, and checked newtypes

When one field needs a different shape, give Serde a function instead of changing the whole type:

```rust
#[serde(serialize_with = "as_percent")]    // only when writing: 0.75 -> "75%"
brightness: f32,
#[serde(deserialize_with = "comma_list")]  // only when reading: "a, b" -> ["a", "b"]
tags: Vec<String>,
#[serde(with = "hex_colour")]              // a module with BOTH: serialize + deserialize
colour: [u8; 3],

fn as_percent<S: Serializer>(value: &f32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{:.0}%", value * 100.0))
}

fn comma_list<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let text = String::deserialize(deserializer)?;   // read something Serde knows...
    Ok(text.split(',').map(str::trim).filter(|t| !t.is_empty()).map(String::from).collect())
}                                                     // ...then convert it
```

`D::Error::custom(message)` (with `use serde::de::Error as _;`) turns your own message into the format's error, so it still gets a position: `expected a colour like "#ff8800", found "teal" at line 5 column 24`.

**`try_from` and `into`** do the same for a whole type, which is how you make invalid values impossible to load:

```rust
#[derive(Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]   // read a String, then Room::try_from(it)
struct Room(String);
```

```text
no room called "attic"; try one of ["kitchen", "hall", "bedroom", "garden"] at line 7 column 5
```

A `with` module is a separate `mod`, and **a module starts with an empty scope**: `String::deserialize` is a method of the `Deserialize` trait, so the module must import it (exercise 3):

```text
error[E0599]: no associated function or constant named `deserialize` found for struct `std::string::String` in the current scope
   --> 05_mastery\07_serialization\exercises\ex03_fix_trait_not_in_scope.rs:37:28
    |
 37 |         let text = String::deserialize(deserializer)?;
    |                            ^^^^^^^^^^^ associated function or constant not found in `std::string::String`
...
    = help: items from traits can only be used if the trait is in scope
help: trait `Deserialize` which provides `deserialize` is implemented but not in scope; perhaps you want to import it
    |
 28 +     use crate::_::_serde::Deserialize;
    |
```

Ignore the path the compiler suggests (`crate::_::_serde` is a hidden detail of the derive macro): write `use serde::Deserialize;`.

> 💡 **Analogy: a special instruction for one piece.** Most pieces follow the normal leaflet, but the castle's flag gets its own little card: "fold it like this when packing, unfold it like this when building" (`with`). And a **bouncer at the door** checks some pieces on the way in: "only these four rooms exist, sorry" (`try_from`).

▶ `cargo run -p serialization --example custom_with`

## 7. A hand-written `Visitor`

The derive covers most types. Write `Deserialize` yourself when a value can arrive in **several shapes**. `Rgb` accepts `"#ff8800"`, `[255, 136, 0]` or `{"r": 255, "g": 136, "b": 0}`:

```rust
impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Rgb, D::Error> {
        deserializer.deserialize_any(RgbVisitor)   // "look, then call the right visit_*"
    }
}

impl<'de> Visitor<'de> for RgbVisitor {
    type Value = Rgb;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a colour: \"#rrggbb\", [r, g, b] or {\"r\", \"g\", \"b\"}")
    }
    fn visit_str<E: de::Error>(self, text: &str) -> Result<Rgb, E> { /* parse hex */ }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Rgb, A::Error> { /* 3 x u8 */ }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Rgb, A::Error> { /* r, g, b */ }
}
```

- The **Deserializer drives**: it reads the input and calls the one `visit_*` method that matches what it found. Every method you don't write returns an "invalid type" error, finished by your `expecting` text.
- The helpers in `serde::de::Error` build the standard messages: `invalid_value`, `invalid_length`, `missing_field`, `unknown_field`, `custom`.
- `deserialize_any` only works with self-describing formats. If your type has one shape, give a **hint** instead (`deserialize_str`, `deserialize_seq`, ...), and postcard can read it too.

```text
  true                               -> error: invalid type: boolean `true`, expected a colour: "#rrggbb", [r, g, b] or {"r", "g", "b"} at line 1 column 4
  [255, 136]                         -> error: invalid length 2, expected a colour: "#rrggbb", [r, g, b] or {"r", "g", "b"} at line 1 column 10
  [300, 0, 0]                        -> error: invalid value: integer `300`, expected u8 at line 1 column 4
  {"r": 1, "g": 2, "b": 3, "a": 4}   -> error: unknown field `a`, expected one of `r`, `g`, `b` at line 1 column 28
...
postcard bytes [7, 35, 102, 102, 56, 56, 48, 48] -> Err(WontImplement)
```

Formats differ in which `visit_*` they call: JSON gives a non-negative integer to `visit_u64`, TOML gives every integer to `visit_i64`. The mini project's `duration` module handles both (exercise 5 too).

> 💡 **Analogy: a parcel sorter at the post office.** You (the Visitor) stand at the end of a conveyor belt with three hatches: one for letters, one for boxes, one for bags. The belt (the Deserializer) looks at each parcel and drops it through the right hatch, and you only have to know what to do with each kind. A parcel with no hatch gets sent back with a note: "expected a letter, a box or a bag".

▶ `cargo run -p serialization --example hand_written_visitor`

## 8. Zero-copy deserialization

A `String` field means an allocation and a copy. A `&'a str` field can **point into the input** instead:

```rust
#[derive(Deserialize)]
struct Song<'a> {
    title: &'a str,     // borrowed from the JSON text: no copy
    artist: &'a str,
    seconds: u32,
}
```

```text
1. Song { title: "Kpanlogo", artist: "Ama", seconds: 214 }
   title points inside the input text: true
4. 1000 songs: borrowed 9 allocations, owned 2009 allocations
```

(The example counts allocations with a tiny `#[global_allocator]`: the owned version allocates two `String`s per song.)

Three things to know:

1. **Escapes can't be borrowed.** `"Tab\there"` in JSON means a real tab character, which isn't in the input bytes, so there is nothing to point at:

   ```text
   2. &str with an escape: invalid type: string "Tab\there", expected a borrowed string at line 1 column 21
   ```

   Use **`Cow<'a, str>` with `#[serde(borrow)]`**: borrowed when possible, an owned `String` when not. Without `#[serde(borrow)]` a `Cow` is always owned.
2. **The input must outlive the struct** (exercise 2):

   ```text
   error[E0597]: `text` does not live long enough
     --> 05_mastery\07_serialization\exercises\ex02_fix_borrow_outlives_buffer.rs:34:30
      |
   32 |     let headline: Headline = {
      |         -------- borrow later stored here
   33 |         let text = fetch();
      |             ---- binding `text` declared here
   34 |         serde_json::from_str(&text).unwrap()
      |                              ^^^^^ borrowed value does not live long enough
   35 |     };
      |     - `text` dropped here while still borrowed
   ```

3. **`Deserialize<'de>` vs `DeserializeOwned`.** `T: Deserialize<'de>` means "T can be built from input that lives for `'de`" (it may borrow). `T: DeserializeOwned` means "from input of *any* lifetime", so it borrows nothing. `from_reader` needs the owned kind, because it reads through a buffer that's thrown away as it goes. Asking it for a `Song` gives an error with no code (verified in a scratch file):

   ```text
   error: implementation of `Deserialize` is not general enough
     |
     = note: `Song<'_>` must implement `Deserialize<'0>`, for any lifetime `'0`...
     = note: ...but `Song<'_>` actually implements `Deserialize<'1>`, for some specific lifetime `'1`
   ```

   And a generic loader with no bound is E0277, whose suggested fix (`T: _::_serde::Deserialize<'_>`) leads into lifetime trouble. Write `T: DeserializeOwned` instead, as the mini project's `Format::parse` does:

   ```text
   error[E0277]: the trait bound `T: serde::Deserialize<'de>` is not satisfied
     |
   28 |     serde_json::from_str(text).unwrap()
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Deserialize<'_>` is not implemented for `T`
   ```

> 💡 **Analogy: pointing instead of copying.** To tell a friend a sentence from a library book, you can copy it out by hand (an owned `String`), or put a bookmark in the book and say "it's there" (a borrowed `&str`). The bookmark is much quicker, but only works while the book is still on the table: give the book back (drop the text) and the bookmark points at nothing. And if the book uses a secret code (an escape), you have to write out the decoded words yourself (`Cow` becomes owned).

▶ `cargo run -p serialization --example zero_copy`

## 9. Dynamic data with `serde_json::Value`

When you don't know the shape in advance (a webhook, a plugin's settings, a quick script), read into `Value`, an enum of `Null`, `Bool`, `Number`, `String`, `Array` and `Object`:

```rust
let mut report = json!({ "cities": [{ "city": "Accra", "celsius": 31.5 }], "updated": null });
report["cities"][0]["city"]            // "Accra": indexing never panics...
report["nope"]["deeper"]               // ...a missing key gives Null
report.get("updated")                  // Some(Null): .get tells "missing" from "null"
report.pointer("/cities/0/celsius")    // a JSON Pointer path
report["updated"] = json!("2026-11-14T09:00Z");
let typed: Weather = serde_json::from_value(report["cities"][0].clone())?;
let back: Value = serde_json::to_value(&typed)?;
let manifest: Value = toml::from_str(cargo_toml)?;   // any self-describing format
```

- `as_str`, `as_f64`, `as_array`, ... return `Option`: `None` if the value is another type.
- `serde_json`'s map keeps keys **sorted** unless you enable its `preserve_order` feature.
- Prefer typed structs when you can: a `Value` defers every mistake to run time. A good middle way is a typed struct with a `#[serde(flatten)] extra: BTreeMap<String, Value>` for the parts you don't know (section 4).

> 💡 **Analogy: a mystery box.** A typed struct is a box with a cut-out shape for every piece. `Value` is a bag you can tip anything into and rummage through later: very flexible, but you only find out the piece is missing when you reach in for it.

▶ `cargo run -p serialization --example dynamic_value`

## 10. Error messages with line and column

```text
1. serde_json errors:
  syntax         Syntax  trailing comma at line 4 column 1
  wrong type     Data    invalid type: string "80", expected u16 at line 3 column 14
  out of range   Data    invalid value: integer `70000`, expected u16 at line 3 column 15
  missing        Data    missing field `port` at line 3 column 1
  unknown field  Data    unknown field `prot`, expected `host` or `port` at line 3 column 8
  cut short      Eof     EOF while parsing an object at line 3 column 11
```

- `serde_json::Error` has `line()`, `column()` (1-based) and `classify()`: `Syntax` (not JSON), `Data` (JSON, wrong shape), `Eof` (cut short), `Io` (the reader failed). Its `Display` already ends with "at line L column C".
- The `toml` crate's `Display` draws the snippet for you, and `span()`/`message()` give the parts:

```text
TOML parse error at line 2, column 8
  |
2 | port = "80"
  |        ^^^^
invalid type: string "80", expected u16
```

- Errors from inside internally tagged and untagged enums have **no position** (line 0, column 0), because Serde buffered the input first.
- The mini project turns both crates' errors into one `ParseError { line, column, message }` and prints a compiler-style snippet for any format.

> 💡 **Analogy: "page 3, line 14".** A teacher who writes "wrong!" on your homework isn't much help. One who writes "page 3, line 14: this should be a number, not a word" lets you fix it in a second.

▶ `cargo run -p serialization --example error_messages`

## 11. Versioning: changing a format without breaking old files

Files outlive programs. A user may open a settings file written by your app three years ago. The safe changes, cheapest first:

| Change | How | Old files still load? |
|---|---|---|
| Add a field | `#[serde(default)]` on it | yes |
| Rename a field | new name + `#[serde(alias = "old_name")]` | yes |
| Remove a field | just remove it (unknown fields are ignored), unless you `deny_unknown_fields` | yes / no |
| Change a field's **meaning or shape** | a new **version**, and a migration step | with migration |

For real changes, the recipe the mini project follows (`src/migrate.rs`):

1. Put a `version` field in every file you write. A file without one is version 1.
2. **Probe**: read only the version, with a tiny struct that ignores everything else.
3. **Parse** the text again as that version's struct. Old versions' structs are **frozen**: they never change again, and only derive `Deserialize`.
4. **Upgrade one step at a time**: `v1_to_v2`, then `v2_to_v3`. Each step is a plain function you can unit-test, and the next version needs just one more.
5. Refuse versions newer than you know, with a clear message.

```rust
#[derive(Deserialize)]
struct Probe { version: Option<u32> }   // NOT deny_unknown_fields: it reads whole files

let settings = match format.parse::<Probe>(text)?.version.unwrap_or(1) {
    1 => v2_to_v3(v1_to_v2(format.parse(text)?, &mut notes)?, &mut notes)?,
    2 => v2_to_v3(format.parse(text)?, &mut notes)?,
    3 => format.parse(text)?,
    v if v > CURRENT_VERSION => return Err(MigrateError::TooNew(v)),
    v => return Err(MigrateError::UnknownVersion(v)),
};
```

An internally tagged enum, `#[serde(tag = "version")]`, looks like a shortcut, but tags must be strings: `"version": 1` fails with ``invalid type: integer `1`, expected variant identifier``.

> 💡 **Analogy: old instruction leaflets.** Your castle kit has changed over the years. When someone hands you an old leaflet, you first look at the **edition number on the cover** (probe), read it with the old rules (frozen structs), then **translate it into the next edition, then the next** (one step at a time), like a translator who only knows neighbouring languages but can pass a message all the way along the line.

## The mini project: a settings loader with migration

`src/` is a settings loader for a pretend backup tool. It reads JSON or TOML, works out the version, upgrades old files step by step, validates the result and writes it back in the current format.

```text
$ cargo run -p serialization
OK    1_v1_family_photos.json    upgraded v1 -> v3 (4 changes)
OK    2_v2_work_laptop.json      upgraded v2 -> v3 (2 changes)
OK    3_v3_current.toml          version 3
FAIL  4_bad_syntax.json          JSON error at line 5, column 1: trailing comma
        |
      5 | }
        | ^
FAIL  5_bad_type.toml            TOML error at line 6, column 8: invalid type: string "9000", expected u16
        |
      6 | port = "9000"
        |        ^
FAIL  6_typo.toml                TOML error at line 6, column 1: unknown field `prot`, expected one of `host`, `port`, `timeout`
        |
      6 | prot = 9000
        | ^
FAIL  7_invalid_values.toml      4 problem(s):
        - keep-versions: must be 1 to 100, not 0
        - server.host: "https://nas.local" is a URL; write only the host name
        - server.timeout: must be between 1s and 10m
        - folders[1].path: "c:/photos" is listed twice
FAIL  8_too_new.toml             version 4 was written by a newer program (this one reads up to 3)
```

```text
$ cargo run -p serialization -- 05_mastery/07_serialization/samples/1_v1_family_photos.json
05_mastery/07_serialization/samples/1_v1_family_photos.json: JSON, version 1
  - v1 -> v2: server_url "backup.example.com:8443" became host "backup.example.com" and port 8443
  - v1 -> v2: nightly = true became schedule "daily@02:00"
  - v2 -> v3: timeout_secs = 20 became timeout = "20s"
  - v2 -> v3: schedule "daily@02:00" became a [schedule] table

As the current version, in TOML:

version = 3
name = "Family photos"
keep-versions = 3

[server]
host = "backup.example.com"
port = 8443
timeout = "20s"

[schedule]
run = "daily"
at = "02:00"

[[folders]]
path = "C:/Users/ama/Pictures"
...
```

| File | Lesson topic |
|---|---|
| [`src/settings.rs`](src/settings.rs) | version 3: `rename_all = "kebab-case"`, `alias`, `default`, `skip_serializing_if`, `deny_unknown_fields`, an internally tagged `Schedule`, `try_from` newtypes `Port` and `TimeOfDay` |
| [`src/duration.rs`](src/duration.rs) | a `with` module and a hand-written `Visitor`: `"1h30m"` or a number of seconds |
| [`src/format.rs`](src/format.rs) | one `parse<T: DeserializeOwned>` for JSON and TOML; `ParseError` with line, column and a snippet |
| [`src/legacy.rs`](src/legacy.rs), [`src/migrate.rs`](src/migrate.rs) | frozen v1 and v2 structs; probe, parse, upgrade |
| [`src/validate.rs`](src/validate.rs) | checks across fields, reporting every problem at once |
| [`samples/`](samples/), [`tests/samples.rs`](tests/samples.rs) | one file per case, and a test that each loads (or fails) as shown above |

Checking happens in **two layers**: the types refuse impossible values *while parsing* (a `Port` can't be 0, a `TimeOfDay` can't be 25:00), with a line and column; `validate` then checks things that need the whole picture (no duplicate folders, a sensible timeout) and lists every problem at once.

> 💡 **Analogy: the removal company.** You're moving house, and boxes arrive from three different years. The movers read the year on each box, unpack old boxes the old way, and **repack everything into this year's boxes**, writing a note for every change ("the 'nightly' label is now a schedule card"). Broken things aren't hidden: they get a list, with exactly which box and which shelf.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `#[derive(Serialize)]` on a struct whose field type has none | E0277 the trait bound `X: serde::Serialize` is not satisfied | Derive it on `X` too, or enable the other crate's `serde` feature. |
| `String::deserialize(d)` inside a `with` module | E0599 no associated function `deserialize` | `use serde::Deserialize;` inside the module (not the compiler's `_::_serde` path). |
| `#[serde(skip)]` on a field whose type has no `Default` | E0277 the trait bound `X: Default` is not satisfied | `impl Default`, or `#[serde(skip, default = "path")]`. |
| `fn load<T>(s: &str) -> T { serde_json::from_str(s)... }` | E0277 `T: serde::Deserialize<'de>` is not satisfied | `T: DeserializeOwned`. |
| A borrowed struct that outlives its input text | E0597 does not live long enough | Keep the text alive longer, or use owned fields. |
| `from_reader` into a struct with `&str` fields | implementation of `Deserialize` is not general enough (no code) | Owned fields (`String`), or read to a `String` first and borrow from it. |
| `&str` field, input contains `\n` or `\"` | invalid type: string ..., expected a borrowed string | `Cow<'a, str>` with `#[serde(borrow)]`. |
| `#[serde(rename_all = "camel")]` | unknown rename rule (from the derive, no code) | Use the exact spelling: `"camelCase"`. |
| `#[serde(tag = "...")]` with a tuple variant | cannot be used with tuple variants (no code) | Use struct variants, or another representation. |
| Integer tag: `#[serde(tag = "version")]` with `"version": 2` | invalid type: integer, expected variant identifier | Probe the version with a struct, then dispatch. |
| A typo in a config file, silently ignored | no error, wrong value | `#[serde(deny_unknown_fields)]` |
| `toml::to_string(&vec![...])` or a `null` | `UnsupportedType` | The top level must be a struct or map; use `Option` and let `None` be left out. |
| `postcard::from_bytes::<serde_json::Value>` or a `deserialize_any` Visitor | `WontImplement` | Non-self-describing formats need concrete types and hints. |
| Changed a struct in place after shipping files | old files fail to load | `default`/`alias` for small changes; a version + migration for big ones. |
| An untagged enum's error says nothing useful | data did not match any variant | Prefer a tagged representation; order variants from most to least specific. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p serialization --example derive_and_json
cargo run -p serialization --example formats
cargo run -p serialization --example field_attributes
cargo run -p serialization --example enum_representations
cargo run -p serialization --example custom_with
cargo run -p serialization --example hand_written_visitor
cargo run -p serialization --example zero_copy
cargo run -p serialization --example dynamic_value
cargo run -p serialization --example error_messages
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_missing_derive` | Make it compile (E0277): a nested type without the derive | `cargo run -p serialization --features exercises --bin ex01_fix_missing_derive` |
| 2 | `ex02_fix_borrow_outlives_buffer` | Make it compile (E0597): zero-copy data must not outlive its input | `cargo run -p serialization --features exercises --bin ex02_fix_borrow_outlives_buffer` |
| 3 | `ex03_fix_trait_not_in_scope` | Make it compile (E0599): a `with` module's missing import | `cargo run -p serialization --features exercises --bin ex03_fix_trait_not_in_scope` |
| 4 | `ex04_match_the_json` | Attributes only: rename, alias, default, skip_serializing_if, tag, pass 3 tests | `cargo test -p serialization --features exercises --bin ex04_match_the_json` |
| 5 | `ex05_hand_written_visitor` | A `Visitor` for numbers and `"70F"` strings, pass 3 tests | `cargo test -p serialization --features exercises --bin ex05_hand_written_visitor` |
| 6 | `ex06_migrate_versions` | Probe, parse and upgrade old files, pass 4 tests | `cargo test -p serialization --features exercises --bin ex06_migrate_versions` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p serialization --features solutions --bins`, or `cargo run -p serialization --features solutions --bin sol01_fix_missing_derive` and so on).

**4. The mini project:** run the loader on every sample and on single files (commands above), then try:
- Add `version = 4` to the code: a new optional field with a default, a `v3_to_v4` step, and a sample file. Which tests needed changing?
- Add a YAML-like third `Format`? (Only if you add a crate for it: notice that `Settings` itself doesn't change at all.)
- `cargo test -p serialization`: 30 unit tests, 5 tests over the sample files and 3 doc tests.

## Checklist

- [ ] How Serde works: `Serialize`/`Serializer`, `Deserialize`/`Deserializer`, the data model
- [ ] Derive, and JSON with `serde_json` (`to_string`, `from_str`, readers and writers)
- [ ] Formats: JSON, TOML, postcard; self-describing or not
- [ ] Field and container attributes: `rename_all`, `rename`, `alias`, `default`, `skip`, `skip_serializing_if`, `flatten`, `deny_unknown_fields`
- [ ] Enum representations: externally, internally, adjacently tagged, untagged, `other`
- [ ] Custom fields: `serialize_with`, `deserialize_with`, `with`, `try_from`/`into`
- [ ] A hand-written `Visitor`, `expecting`, and `de::Error` helpers
- [ ] Zero-copy: `&'de str`, `Cow` with `#[serde(borrow)]`, `DeserializeOwned`
- [ ] Dynamic data with `serde_json::Value` and `json!`
- [ ] Error messages with line and column
- [ ] Versioning and migration: probe, frozen structs, one step at a time
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- Serde splits (de)serialization into what your type is (`Serialize`, `Deserialize`) and how a format writes it (`Serializer`, `Deserializer`), meeting in a shared data model. Derive the traits once and every format works.
- Attributes reshape the data: names, defaults, skipped and flattened fields, strictness, and four ways to write an enum.
- For one odd field use `with`; for checked values use `try_from`; for several input shapes write a `Visitor`.
- Borrowed fields (`&'de str`, `Cow`) avoid copies but tie the result to the input; `DeserializeOwned` means "borrows nothing".
- `Value` handles unknown shapes, good errors carry a line and column, and versioned files are read by probing, parsing the old struct and upgrading step by step.

**In one sentence for a child:** Serde is the instruction leaflet that lets you take your toy castle apart, post it in any kind of box, and have your friend build exactly the same castle again, even from a leaflet written years ago.

**Next:** [08 `no_std` and embedded](../08_no_std_and_embedded/README.md)
