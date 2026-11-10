# 06 Framework Mastery

Real-world crates and frameworks, in the order to learn them.

1. **Axum** (web, built on Tokio + Tower): routing, extractors, state, middleware, error handling, JSON with Serde, testing. Start in `axum_basics`.
2. **Tokio** (async runtime): tasks, channels, select!, timeouts, streams.
3. **Serde** (serialization): derive, custom (de)serializers, formats.
4. **SQLx** (async SQL, compile-time checked queries) with Postgres/SQLite; migrations; Axum + SQLx REST API.
5. **Clap** (CLI): derive API, subcommands, config.
6. **Actix-web** (the other popular web framework): compare its design with Axum.
7. **Tauri** (desktop apps) and **Bevy** (games), as optional later tracks.

Capstone: a full REST API with auth, database, tests and Docker.
