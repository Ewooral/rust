//! Exercise 1: make it compile (E0038).
//!
//! A help desk asks several user services in turn, and which services it
//! has is decided at run time, so it keeps them in a
//! `Vec<Box<dyn UserService>>`. But `UserService` uses a native `async fn`,
//! and a trait with an `async fn` is not dyn compatible.
//!
//! Change ONLY the trait and the two impls, not `main`:
//!   - make `get_user` a plain `fn` returning
//!     `Pin<Box<dyn Future<Output = Option<User>> + '_>>`
//!   - wrap each body in `Box::pin(async move { … })`.
//!
//! (Nothing here is spawned, so `+ Send` isn't needed; adding it is fine.)
//!
//! Run: cargo run -p async_traits_from_scratch --features exercises --bin ex01_fix_dyn_compatible

use std::collections::HashMap;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

trait UserService {
    async fn get_user(&self, id: u32) -> Option<User>;
}

/// The staff list: a table.
struct Staff {
    users: HashMap<u32, User>,
}

impl UserService for Staff {
    async fn get_user(&self, id: u32) -> Option<User> {
        tokio::task::yield_now().await; // pretend: a database round trip
        self.users.get(&id).cloned()
    }
}

/// Visitors get ids from 100 up, and anyone with such an id is welcome.
struct Visitors;

impl UserService for Visitors {
    async fn get_user(&self, id: u32) -> Option<User> {
        (id >= 100).then(|| User::new(id, format!("visitor #{id}")))
    }
}

#[tokio::main]
async fn main() {
    let desks: Vec<Box<dyn UserService>> = vec![
        Box::new(Staff {
            users: HashMap::from([(1, User::new(1, "Ada"))]),
        }),
        Box::new(Visitors),
    ];

    for id in [1, 100, 7] {
        let mut answer = None;
        for desk in &desks {
            if let Some(user) = desk.get_user(id).await {
                answer = Some(user);
                break;
            }
        }
        match answer {
            Some(user) => println!("{id}: {} (id {})", user.name, user.id),
            None => println!("{id}: nobody at any desk"),
        }
    }
}
