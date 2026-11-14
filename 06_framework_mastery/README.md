# 06 Framework Mastery

Real-world crates and frameworks, in the order to learn them. Every lesson follows the course format: an analogy for every concept, runnable examples, exercises behind `--features exercises`, solutions behind `--features solutions`, and a mini project with tests that need no network or database server.

| Lesson | Package | What you learn |
|---|---|---|
| [01 Axum](01_axum/README.md) | `axum_basics` | Routing, extractors, responses, shared state, `IntoResponse` errors, tower middleware, graceful shutdown, testing with `oneshot` |
| [02 Tokio](02_tokio/README.md) | `tokio_runtime` | The runtime, tasks and `JoinSet`, sync primitives, channels, `select!` and cancellation, async I/O, streams, shutdown |
| [03 SQLx](03_sqlx/README.md) | `sqlx_database` | Pools, bound parameters, `FromRow`, transactions, migrations, compile-time checked queries (SQLite, so no server needed) |
| [04 Clap](04_clap/README.md) | `clap_cli` | The derive API, subcommands, env and defaults, value parsers, completions, exit codes, testing CLIs |
| [05 Actix-web](05_actix_web/README.md) | `actix_web_basics` | The other big web framework, side by side with Axum: extractors, `web::Data`, `ResponseError`, middleware, testing |
| [06 Tauri and Bevy](06_tauri_and_bevy/README.md) | `tauri_and_bevy` | Bevy's ECS and Tauri's command/IPC model, rebuilt in plain Rust; real project setup explained |
| [07 Capstone: a REST API](07_capstone_rest_api/README.md) | `capstone_rest_api` | The course finale: Axum + SQLx + Tokio, Argon2id passwords, JWT auth, authorisation, pagination, logging, Docker |

**Serde** is taught in [05 Mastery / 07 Serialization](../05_mastery/07_serialization/README.md), and every lesson here uses it.

**Tauri and Bevy** are big to build, so lesson 06 teaches their designs with small plain-Rust versions, and gives exact steps for starting real projects.

**Next:** this is the last category. The capstone's README ends the course and points back to the [course map](../README.md).
