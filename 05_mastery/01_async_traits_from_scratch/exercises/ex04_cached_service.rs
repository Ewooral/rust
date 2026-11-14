//! Exercise 4: implement a caching layer by hand (pass 3 tests).
//!
//! `CachedUsers` wraps any other `Arc<dyn UserService>` and remembers the
//! users it has found. Implement its `get_user`, in the hand-written style
//! of steps 3 to 6 (no `#[async_trait]`):
//!
//!   1. Look in `self.cache`. If the user is there, return a clone.
//!   2. Otherwise ask `self.inner`, and if it found the user, store a clone
//!      in the cache before returning it.
//!   3. Don't cache "not found": that user might be added later.
//!
//! The body is `Box::pin(async move { … })`. Careful with the lock: a std
//! `MutexGuard` must not be alive across an `.await`, or the future isn't
//! `Send` and won't fit `BoxFuture` ("future cannot be sent between threads
//! safely"). Copy what you need out of the cache in one statement, so the
//! guard is dropped at its `;`.
//!
//! Test: cargo test -p async_traits_from_scratch --features exercises --bin ex04_cached_service

#![allow(unused_variables, dead_code)]

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_traits_from_scratch::User; // { id, name }: the mini project's user

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait UserService: Send + Sync {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

/// A pretend database that counts how often it's asked.
struct Database {
    users: HashMap<u32, User>,
    lookups: AtomicUsize,
}

impl Database {
    fn new() -> Self {
        Database {
            users: HashMap::from([(1, User::new(1, "Ada"))]),
            lookups: AtomicUsize::new(0),
        }
    }
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move {
            self.lookups.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await; // pretend: a slow round trip
            self.users.get(&id).cloned()
        })
    }
}

/// Remembers every user it has found.
struct CachedUsers {
    inner: Arc<dyn UserService>,
    cache: Mutex<HashMap<u32, User>>,
}

impl CachedUsers {
    fn new(inner: Arc<dyn UserService>) -> Self {
        CachedUsers {
            inner,
            cache: Mutex::new(HashMap::new()),
        }
    }
}

impl UserService for CachedUsers {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let database = Arc::new(Database::new());
    let cached = CachedUsers::new(database.clone());
    for _ in 0..3 {
        println!("{:?}", cached.get_user(1).await);
    }
    println!(
        "the database was asked {} time(s)",
        database.lookups.load(Ordering::SeqCst)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn repeat_lookups_come_from_the_cache() {
        let database = Arc::new(Database::new());
        let cached = CachedUsers::new(database.clone());
        for _ in 0..3 {
            assert_eq!(cached.get_user(1).await.unwrap().name, "Ada");
        }
        assert_eq!(database.lookups.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn not_found_is_not_cached() {
        let database = Arc::new(Database::new());
        let cached = CachedUsers::new(database.clone());
        assert_eq!(cached.get_user(42).await, None);
        assert_eq!(cached.get_user(42).await, None);
        assert_eq!(database.lookups.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn the_cache_works_behind_dyn_from_spawned_tasks() {
        let database = Arc::new(Database::new());
        let cached: Arc<dyn UserService> = Arc::new(CachedUsers::new(database.clone()));
        assert!(cached.get_user(1).await.is_some()); // fill the cache first
        let tasks: Vec<_> = (0..4)
            .map(|_| {
                let cached = Arc::clone(&cached);
                tokio::spawn(async move { cached.get_user(1).await })
            })
            .collect();
        for task in tasks {
            assert_eq!(task.await.unwrap().unwrap().id, 1);
        }
        assert_eq!(database.lookups.load(Ordering::SeqCst), 1);
    }
}
