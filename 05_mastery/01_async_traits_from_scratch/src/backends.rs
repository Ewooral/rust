//! Two backends: a fast in-memory table and a slow "remote" one that can go down.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;

use crate::service::{BoxFuture, LookupError, User, UserService};

/// Users kept in a `HashMap`: answers at once.
pub struct InMemoryUsers {
    users: HashMap<u32, User>,
}

impl InMemoryUsers {
    /// Makes a table from any list of users.
    pub fn new(users: impl IntoIterator<Item = User>) -> Self {
        InMemoryUsers {
            users: users.into_iter().map(|user| (user.id, user)).collect(),
        }
    }
}

/// Three famous programmers, with ids 1, 2 and 3.
pub fn sample_users() -> Vec<User> {
    vec![
        User::new(1, "Ada Lovelace"),
        User::new(2, "Grace Hopper"),
        User::new(3, "Alan Turing"),
    ]
}

// This impl is written WITHOUT `#[async_trait]`, using the exact signature
// the macro generates. It shows the macro is "only" boxing: a hand-written
// impl and a macro-written impl fit the same trait.
impl UserService for InMemoryUsers {
    fn get_user<'life0, 'async_trait>(
        &'life0 self,
        id: u32,
    ) -> BoxFuture<'async_trait, Result<User, LookupError>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            self.users
                .get(&id)
                .cloned()
                .ok_or(LookupError::NotFound(id))
        })
    }

    fn describe(&self) -> String {
        String::from("memory")
    }
}

/// Pretends to be a user database across a network: every lookup waits for
/// `latency`, and the whole service can be switched off.
pub struct RemoteUsers {
    users: HashMap<u32, User>,
    latency: Duration,
    up: AtomicBool,
    calls: AtomicUsize,
}

impl RemoteUsers {
    /// A remote service holding `users`, answering after `latency`.
    pub fn new(users: impl IntoIterator<Item = User>, latency: Duration) -> Self {
        RemoteUsers {
            users: users.into_iter().map(|user| (user.id, user)).collect(),
            latency,
            up: AtomicBool::new(true),
            calls: AtomicUsize::new(0),
        }
    }

    /// Switches the service on or off. An atomic lets this happen through a
    /// shared `&self`, even while other tasks are using the service.
    pub fn set_up(&self, up: bool) {
        self.up.store(up, Ordering::Relaxed);
    }

    /// How many lookups reached this service so far.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl UserService for RemoteUsers {
    async fn get_user(&self, id: u32) -> Result<User, LookupError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        tokio::time::sleep(self.latency).await; // pretend: a network round trip
        if !self.up.load(Ordering::Relaxed) {
            return Err(LookupError::Unavailable(self.describe()));
        }
        self.users
            .get(&id)
            .cloned()
            .ok_or(LookupError::NotFound(id))
    }

    fn describe(&self) -> String {
        format!("remote({}ms)", self.latency.as_millis())
    }
}
