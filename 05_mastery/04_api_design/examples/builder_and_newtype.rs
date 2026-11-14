//! Builders (two styles) and newtypes (type safety, validation, hiding the
//! implementation).
//!
//! Run: cargo run -p api_design --example builder_and_newtype

use std::fmt;

// --- Newtypes for type safety -------------------------------------------
// Two ids that are both `u64` underneath. As plain `u64`s they're easy to
// swap by accident; as newtypes, swapping them doesn't compile:
//     cancel(OrderId(7), UserId(1));
//     error[E0308]: arguments to this function are incorrect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OrderId(u64);

fn cancel(user: UserId, order: OrderId) -> String {
    format!("user {} cancelled order {}", user.0, order.0)
}

// --- Newtypes for validation ----------------------------------------------
/// A port a normal program may listen on. The field is private, so the
/// only way to make one is `Port::new`, which checks: every `Port` is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Port(u16);

impl Port {
    fn new(n: u16) -> Result<Port, String> {
        if n < 1024 {
            Err(format!("port {n} is reserved for the system"))
        } else {
            Ok(Port(n))
        }
    }

    fn get(self) -> u16 {
        self.0
    }
}

// --- A consuming builder: `self -> Self` -----------------------------------
// Chains in one expression and can MOVE values into the result. The style
// of most builders (and of `api_design::RequestBuilder`).
#[derive(Debug)]
struct Server {
    name: String,
    port: Port,
    workers: usize,
    tls: bool,
}

#[derive(Debug, Default)]
#[must_use = "a builder does nothing until you call `.build()`"]
struct ServerBuilder {
    name: Option<String>,
    port: Option<u16>,
    workers: Option<usize>,
    tls: bool,
}

impl ServerBuilder {
    fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn workers(mut self, workers: usize) -> Self {
        self.workers = Some(workers);
        self
    }

    fn tls(mut self, on: bool) -> Self {
        self.tls = on;
        self
    }

    /// Defaults and validation happen in ONE place.
    fn build(self) -> Result<Server, String> {
        Ok(Server {
            name: self.name.ok_or("a server needs a name")?,
            port: Port::new(self.port.unwrap_or(8080))?,
            workers: self.workers.unwrap_or(4),
            tls: self.tls,
        })
    }
}

// --- A non-consuming builder: `&mut self -> &mut Self` --------------------
// Like `std::process::Command`: the builder can be configured in steps,
// in `if`s, and reused for several results.
#[derive(Debug, Default)]
struct QueryBuilder {
    table: String,
    filters: Vec<String>,
    limit: Option<usize>,
}

impl QueryBuilder {
    fn new(table: &str) -> Self {
        QueryBuilder {
            table: table.into(),
            ..Default::default()
        }
    }

    fn filter(&mut self, condition: &str) -> &mut Self {
        self.filters.push(condition.into());
        self
    }

    fn limit(&mut self, n: usize) -> &mut Self {
        self.limit = Some(n);
        self
    }

    fn build(&self) -> Query {
        let mut sql = format!("SELECT * FROM {}", self.table);
        if !self.filters.is_empty() {
            sql += &format!(" WHERE {}", self.filters.join(" AND "));
        }
        if let Some(n) = self.limit {
            sql += &format!(" LIMIT {n}");
        }
        Query(sql)
    }
}

// --- A newtype that hides the implementation ------------------------------
/// The query text. Users get `Display`; nobody can tell (or depend on the
/// fact) that it's a `String` inside, so it could become something else.
struct Query(String);

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn main() {
    println!("{}", cancel(UserId(1), OrderId(7)));

    println!("Port::new(80)   = {:?}", Port::new(80));
    println!("Port::new(8080) = {:?}", Port::new(8080).map(Port::get));

    let server = ServerBuilder::default()
        .name("api")
        .port(8443)
        .workers(8)
        .tls(true)
        .build();
    println!("\nconsuming builder: {server:?}");
    if let Ok(server) = &server {
        println!(
            "  {} on port {} with {} workers, tls {}",
            server.name,
            server.port.get(),
            server.workers,
            server.tls
        );
    }
    println!(
        "no name:  {:?}",
        ServerBuilder::default().port(9000).build()
    );
    println!(
        "bad port: {:?}",
        ServerBuilder::default().name("x").port(22).build()
    );

    let mut users = QueryBuilder::new("users");
    users.filter("age > 30");
    let all_adults = users.build(); // build from a borrow...
    users.filter("city = 'Accra'").limit(10);
    let some_in_accra = users.build(); // ...and again, after more steps
    println!("\nnon-consuming builder:\n  {all_adults}\n  {some_in_accra}");
}
