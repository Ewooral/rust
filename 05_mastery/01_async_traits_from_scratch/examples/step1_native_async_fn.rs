//! Step 1: a native `async fn` in a trait. It works with generics, but the
//! trait can't be used as `dyn UserService`.
//!
//! Run: cargo run -p async_traits_from_scratch --example step1_native_async_fn

use std::collections::HashMap;
use std::time::Duration;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

trait UserService {
    async fn get_user(&self, id: u32) -> Option<User>;
}

/// A "database" in a HashMap.
struct Database {
    users: HashMap<u32, User>,
}

impl UserService for Database {
    async fn get_user(&self, id: u32) -> Option<User> {
        tokio::time::sleep(Duration::from_millis(5)).await; // pretend: a round trip
        self.users.get(&id).cloned()
    }
}

/// Makes up a guest for any id. Its future keeps more data across the
/// `.await`, so it is a different size from `Database`'s.
struct Guests;

impl UserService for Guests {
    async fn get_user(&self, id: u32) -> Option<User> {
        let name = format!("guest #{id}");
        let badge = [0u8; 64]; // pretend: something big we still need after the await
        tokio::time::sleep(Duration::from_millis(5)).await;
        Some(User::new(id, format!("{name} (badge {})", badge.len())))
    }
}

// Static dispatch: the compiler writes one copy of `greet` per service type,
// and in each copy it knows the exact future `get_user` returns.
async fn greet(service: &impl UserService, id: u32) -> String {
    match service.get_user(id).await {
        Some(user) => format!("Hello, {} (id {})!", user.name, user.id),
        None => format!("Nobody has id {id}."),
    }
}

#[tokio::main]
async fn main() {
    let database = Database {
        users: HashMap::from([(1, User::new(1, "Ada"))]),
    };
    println!("{}", greet(&database, 1).await);
    println!("{}", greet(&database, 2).await);
    println!("{}", greet(&Guests, 7).await);

    // Why no `dyn`? Every impl's `async fn` returns its OWN hidden future
    // type, with its own size. A vtable needs one return type for the slot.
    // (Creating a future runs nothing: futures are lazy.)
    println!(
        "\nDatabase's future is {} bytes, Guests' future is {} bytes",
        std::mem::size_of_val(&database.get_user(1)),
        std::mem::size_of_val(&Guests.get_user(1)),
    );

    // So choosing a service at run time doesn't compile:
    // let services: Vec<Box<dyn UserService>> = vec![Box::new(database), Box::new(Guests)];
    // ^ E0038: the trait `UserService` is not dyn compatible
    //          ...because method `get_user` is `async`
}
