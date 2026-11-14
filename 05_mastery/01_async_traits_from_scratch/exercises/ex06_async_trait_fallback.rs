//! Exercise 6: a fallback layer with `#[async_trait]` (pass 3 tests).
//!
//! With `#[async_trait]` you write plain `async fn`s again, and the macro
//! writes the `Pin<Box<dyn Future + Send + '…>>` and the lifetimes. Even
//! `find_by_name`, which borrows TWO things (`&self` and `name`) across an
//! `.await`, needs no lifetime annotations.
//!
//! Implement both methods of `Fallback`:
//!   - ask `primary` first;
//!   - if it answers `Err(LookupError::Unavailable)`, ask `secondary`;
//!   - any other answer (a user, or `Err(NotFound)`) is final: don't ask
//!     `secondary`, it would only waste time.
//!
//! Test: cargo test -p async_traits_from_scratch --features exercises --bin ex06_async_trait_fallback

#![allow(unused_variables, dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use async_trait::async_trait;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

#[derive(Debug, Clone, PartialEq)]
enum LookupError {
    NotFound,
    Unavailable,
}

#[async_trait]
trait UserService: Send + Sync {
    async fn get_user(&self, id: u32) -> Result<User, LookupError>;
    async fn find_by_name(&self, name: &str) -> Result<User, LookupError>;
}

/// A directory that can go down, and counts the questions it gets.
struct Directory {
    users: Vec<User>,
    up: AtomicBool,
    asked: AtomicUsize,
}

impl Directory {
    fn new(names: &[&str]) -> Self {
        Directory {
            users: (1..)
                .zip(names)
                .map(|(id, name)| User::new(id, *name))
                .collect(),
            up: AtomicBool::new(true),
            asked: AtomicUsize::new(0),
        }
    }

    async fn answer(&self, wanted: impl Fn(&User) -> bool) -> Result<User, LookupError> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await; // pretend: a network round trip
        if !self.up.load(Ordering::SeqCst) {
            return Err(LookupError::Unavailable);
        }
        self.users
            .iter()
            .find(|user| wanted(user))
            .cloned()
            .ok_or(LookupError::NotFound)
    }
}

#[async_trait]
impl UserService for Directory {
    async fn get_user(&self, id: u32) -> Result<User, LookupError> {
        self.answer(|user| user.id == id).await
    }

    async fn find_by_name(&self, name: &str) -> Result<User, LookupError> {
        self.answer(|user| user.name == name).await
    }
}

/// Asks `primary`, and `secondary` only when `primary` is unavailable.
struct Fallback {
    primary: Arc<dyn UserService>,
    secondary: Arc<dyn UserService>,
}

#[async_trait]
impl UserService for Fallback {
    async fn get_user(&self, id: u32) -> Result<User, LookupError> {
        todo!()
    }

    async fn find_by_name(&self, name: &str) -> Result<User, LookupError> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let primary = Arc::new(Directory::new(&["Ada", "Grace"]));
    let secondary = Arc::new(Directory::new(&["Ada (backup)", "Grace (backup)"]));
    let service = Fallback {
        primary: primary.clone(),
        secondary,
    };
    println!("{:?}", service.get_user(1).await);
    println!("{:?}", service.find_by_name("Grace").await);
    primary.up.store(false, Ordering::SeqCst); // now the backup answers
    println!("{:?}", service.get_user(1).await);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Arc<Directory>, Arc<Directory>, Arc<dyn UserService>) {
        let primary = Arc::new(Directory::new(&["Ada", "Grace"]));
        let secondary = Arc::new(Directory::new(&["Ada", "Grace", "Linus"]));
        let fallback = Arc::new(Fallback {
            primary: primary.clone(),
            secondary: secondary.clone(),
        });
        (primary, secondary, fallback)
    }

    #[tokio::test]
    async fn the_primary_answers_when_it_is_up() {
        let (primary, secondary, service) = setup();
        assert_eq!(service.get_user(2).await.unwrap().name, "Grace");
        assert_eq!(service.find_by_name("Ada").await.unwrap().id, 1);
        assert_eq!(primary.asked.load(Ordering::SeqCst), 2);
        assert_eq!(secondary.asked.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn the_secondary_answers_when_the_primary_is_down() {
        let (primary, secondary, service) = setup();
        primary.up.store(false, Ordering::SeqCst);
        assert_eq!(service.get_user(3).await.unwrap().name, "Linus");
        let name = String::from("Grace"); // a borrowed &str, held across .awaits
        assert_eq!(service.find_by_name(&name).await.unwrap().id, 2);
        assert_eq!(secondary.asked.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn not_found_is_final_and_works_from_a_spawned_task() {
        let (_primary, secondary, service) = setup();
        let task = tokio::spawn(async move { service.get_user(3).await });
        assert_eq!(task.await.unwrap(), Err(LookupError::NotFound));
        assert_eq!(secondary.asked.load(Ordering::SeqCst), 0);
    }
}
