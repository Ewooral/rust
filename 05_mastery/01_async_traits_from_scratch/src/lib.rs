//! # A user directory with swappable, layered backends
//!
//! The mini project for "Async traits from scratch". It is the finished
//! `UserService` trait from the lesson (written with `#[async_trait]`), two
//! backends, and three layers that each wrap any other service:
//!
//! ```text
//! cache ──▶ fallback ──┬──▶ timeout ──▶ remote   (slow, can go down)
//!                      └──▶ memory               (fast, a local copy)
//! ```
//!
//! Every piece is an `Arc<dyn UserService>`, so the stack is put together at
//! run time ([`build_stack`]) and shared by many spawned tasks
//! ([`lookup_many`]).
//!
//! ```
//! use std::sync::Arc;
//! use async_traits_from_scratch::{InMemoryUsers, UserService, User, sample_users};
//!
//! let service: Arc<dyn UserService> = Arc::new(InMemoryUsers::new(sample_users()));
//! let runtime = tokio::runtime::Runtime::new().unwrap();
//! let ada = runtime.block_on(service.get_user(1)).unwrap();
//! assert_eq!(ada, User::new(1, "Ada Lovelace"));
//! ```

pub mod backends;
pub mod layers;
pub mod service;
pub mod stack;

pub use backends::{InMemoryUsers, RemoteUsers, sample_users};
pub use layers::{CachedUsers, FallbackUsers, TimeoutUsers};
pub use service::{BoxFuture, LookupError, User, UserService};
pub use stack::{PRESETS, build_stack, lookup_many};

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    fn remote(latency_ms: u64) -> Arc<RemoteUsers> {
        Arc::new(RemoteUsers::new(
            sample_users(),
            Duration::from_millis(latency_ms),
        ))
    }

    fn memory() -> Arc<InMemoryUsers> {
        Arc::new(InMemoryUsers::new(sample_users()))
    }

    #[test]
    fn the_trait_object_is_send_and_sync() {
        // Checked at compile time: this only builds if it's true.
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn UserService>();
        assert_send_sync::<Arc<dyn UserService>>();
    }

    #[tokio::test]
    async fn the_hand_written_impl_works_behind_dyn() {
        let service: Arc<dyn UserService> = memory();
        assert_eq!(service.get_user(2).await, Ok(User::new(2, "Grace Hopper")));
        assert_eq!(service.get_user(9).await, Err(LookupError::NotFound(9)));
    }

    #[tokio::test]
    async fn remote_reports_unavailable_when_down() {
        let remote = remote(1);
        remote.set_up(false);
        let answer = remote.get_user(1).await;
        assert_eq!(answer, Err(LookupError::Unavailable("remote(1ms)".into())));
        assert_eq!(remote.calls(), 1);
    }

    #[tokio::test]
    async fn the_cache_answers_repeat_lookups_itself() {
        let remote = remote(1);
        let cache = CachedUsers::new(remote.clone());
        assert!(cache.get_user(1).await.is_ok());
        assert!(cache.get_user(1).await.is_ok());
        assert!(cache.get_user(1).await.is_ok());
        assert_eq!(remote.calls(), 1, "only the first lookup reaches remote");
        assert_eq!(cache.hits(), 2);
    }

    #[tokio::test]
    async fn the_cache_does_not_store_errors() {
        let remote = remote(1);
        let cache = CachedUsers::new(remote.clone());
        assert_eq!(cache.get_user(42).await, Err(LookupError::NotFound(42)));
        assert_eq!(cache.get_user(42).await, Err(LookupError::NotFound(42)));
        assert_eq!(remote.calls(), 2);
        assert_eq!(cache.hits(), 0);
    }

    #[tokio::test]
    async fn a_time_limit_turns_a_slow_backend_into_an_error() {
        // 10 seconds of latency, but the test ends after 20 ms: the timed-out
        // future is dropped, and a dropped future stops running.
        let slow = TimeoutUsers::new(remote(10_000), Duration::from_millis(20));
        assert_eq!(
            slow.get_user(1).await,
            Err(LookupError::TimedOut("remote(10000ms)".into()))
        );
    }

    #[tokio::test]
    async fn fallback_asks_the_secondary_only_when_the_primary_fails() {
        let remote = remote(1);
        let fallback = FallbackUsers::new(remote.clone(), memory());

        assert_eq!(fallback.get_user(1).await.unwrap().name, "Ada Lovelace");
        // A real "not found" is an answer, so the secondary isn't asked.
        assert_eq!(fallback.get_user(42).await, Err(LookupError::NotFound(42)));

        remote.set_up(false);
        assert_eq!(fallback.get_user(3).await.unwrap().name, "Alan Turing");
        assert_eq!(remote.calls(), 3);
    }

    #[tokio::test]
    async fn the_resilient_stack_survives_a_remote_outage() {
        let remote = remote(1);
        remote.set_up(false);
        let stack = build_stack(
            "resilient",
            remote.clone(),
            memory(),
            Duration::from_secs(1),
        )
        .expect("a known preset");
        assert_eq!(
            stack.describe(),
            "cache(fallback(timeout(remote(1ms), 1000ms) or memory))"
        );
        assert_eq!(stack.get_user(2).await.unwrap().name, "Grace Hopper");
        assert_eq!(stack.get_user(2).await.unwrap().name, "Grace Hopper");
        assert_eq!(remote.calls(), 1, "the second lookup came from the cache");
    }

    #[test]
    fn build_stack_knows_every_preset_and_nothing_else() {
        for preset in PRESETS {
            assert!(build_stack(preset, remote(1), memory(), Duration::from_secs(1)).is_some());
        }
        assert!(build_stack("turbo", remote(1), memory(), Duration::from_secs(1)).is_none());
    }

    /// Counts how many lookups are running at the same moment.
    #[derive(Default)]
    struct Tracking {
        now: AtomicUsize,
        most: AtomicUsize,
    }

    #[async_trait]
    impl UserService for Tracking {
        async fn get_user(&self, id: u32) -> Result<User, LookupError> {
            let now = self.now.fetch_add(1, Ordering::SeqCst) + 1;
            self.most.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(20)).await;
            self.now.fetch_sub(1, Ordering::SeqCst);
            Ok(User::new(id, format!("user {id}")))
        }

        fn describe(&self) -> String {
            String::from("tracking")
        }
    }

    #[tokio::test]
    async fn lookup_many_runs_concurrently_and_keeps_the_order() {
        let tracking = Arc::new(Tracking::default());
        let answers = lookup_many(tracking.clone(), &[5, 3, 8, 1]).await;
        let ids: Vec<u32> = answers.into_iter().map(|a| a.unwrap().id).collect();
        assert_eq!(ids, [5, 3, 8, 1]);
        assert!(
            tracking.most.load(Ordering::SeqCst) > 1,
            "the lookups should overlap"
        );
    }
}
