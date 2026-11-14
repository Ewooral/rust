//! Solution to exercise 3: make it compile (E0521: borrowed data escapes outside of function).
//!
//! `spawn_lookup` starts a lookup in the background. The trait is already
//! dyn compatible and `Send + Sync`. But the future `get_user` returns
//! BORROWS the service (its type says `BoxFuture<'_, …>`), and
//! `tokio::spawn` only accepts `'static` futures: the task might still be
//! running after `spawn_lookup` returns and the borrow has ended.
//!
//! Don't change the trait or `Database`. Change `spawn_lookup` and `main`:
//!   - take `service: Arc<dyn UserService>` instead of `&dyn UserService`
//!   - spawn `async move { service.get_user(id).await }`, so the task OWNS
//!     its Arc and borrows from that
//!   - in `main`, put the database in an `Arc` and pass `Arc::clone(&…)`.
//!
//! Answer: the task now OWNS an `Arc`, so it is `'static`. The borrow that
//! `get_user` makes is a borrow of the task's own Arc, which lives exactly
//! as long as the task.
//!
//! Run: cargo run -p async_traits_from_scratch --features solutions --bin sol03_fix_spawn_borrowed

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinHandle;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait UserService: Send + Sync {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

struct Database {
    users: HashMap<u32, User>,
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            self.users.get(&id).cloned()
        })
    }
}

/// Starts looking `id` up in the background, and returns at once.
fn spawn_lookup(service: Arc<dyn UserService>, id: u32) -> JoinHandle<Option<User>> {
    tokio::spawn(async move { service.get_user(id).await })
}

#[tokio::main]
async fn main() {
    let database: Arc<dyn UserService> = Arc::new(Database {
        users: HashMap::from([(1, User::new(1, "Ada")), (2, User::new(2, "Grace"))]),
    });

    let handles: Vec<_> = [1, 2, 3]
        .into_iter()
        .map(|id| spawn_lookup(Arc::clone(&database), id))
        .collect();
    for handle in handles {
        match handle.await.unwrap() {
            Some(user) => println!("found {} (id {})", user.name, user.id),
            None => println!("not found"),
        }
    }
}
