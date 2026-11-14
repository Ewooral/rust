//! Layers: services that wrap another `Arc<dyn UserService>` and add one job.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use crate::service::{LookupError, User, UserService};

/// Remembers every user it has found, and answers repeat lookups itself.
pub struct CachedUsers {
    inner: Arc<dyn UserService>,
    cache: Mutex<HashMap<u32, User>>,
    hits: AtomicUsize,
}

impl CachedUsers {
    /// Puts a cache in front of `inner`.
    pub fn new(inner: Arc<dyn UserService>) -> Self {
        CachedUsers {
            inner,
            cache: Mutex::new(HashMap::new()),
            hits: AtomicUsize::new(0),
        }
    }

    /// How many lookups the cache answered without asking `inner`.
    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl UserService for CachedUsers {
    async fn get_user(&self, id: u32) -> Result<User, LookupError> {
        // The guard is dropped at the end of this statement, BEFORE the
        // `.await` below. A std `MutexGuard` held across an `.await` would
        // make the future non-`Send`, and `#[async_trait]` would refuse it.
        let cached = self.cache.lock().unwrap().get(&id).cloned();
        if let Some(user) = cached {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(user);
        }
        let user = self.inner.get_user(id).await?; // errors are not cached
        self.cache.lock().unwrap().insert(id, user.clone());
        Ok(user)
    }

    fn describe(&self) -> String {
        format!("cache({})", self.inner.describe())
    }
}

/// Gives up on `inner` after `limit`.
pub struct TimeoutUsers {
    inner: Arc<dyn UserService>,
    limit: Duration,
}

impl TimeoutUsers {
    /// Wraps `inner` with a time limit.
    pub fn new(inner: Arc<dyn UserService>, limit: Duration) -> Self {
        TimeoutUsers { inner, limit }
    }
}

#[async_trait]
impl UserService for TimeoutUsers {
    async fn get_user(&self, id: u32) -> Result<User, LookupError> {
        // `timeout` drops (cancels) the inner future when time runs out.
        match tokio::time::timeout(self.limit, self.inner.get_user(id)).await {
            Ok(answer) => answer,
            Err(_elapsed) => Err(LookupError::TimedOut(self.inner.describe())),
        }
    }

    fn describe(&self) -> String {
        format!(
            "timeout({}, {}ms)",
            self.inner.describe(),
            self.limit.as_millis()
        )
    }
}

/// Asks `primary` first, and `secondary` only when `primary` is down or slow.
pub struct FallbackUsers {
    primary: Arc<dyn UserService>,
    secondary: Arc<dyn UserService>,
}

impl FallbackUsers {
    /// Tries `primary`, then `secondary`.
    pub fn new(primary: Arc<dyn UserService>, secondary: Arc<dyn UserService>) -> Self {
        FallbackUsers { primary, secondary }
    }
}

#[async_trait]
impl UserService for FallbackUsers {
    async fn get_user(&self, id: u32) -> Result<User, LookupError> {
        match self.primary.get_user(id).await {
            Err(error) if error.is_retryable() => self.secondary.get_user(id).await,
            answer => answer, // found, or a real "not found": don't ask again
        }
    }

    fn describe(&self) -> String {
        format!(
            "fallback({} or {})",
            self.primary.describe(),
            self.secondary.describe()
        )
    }
}
