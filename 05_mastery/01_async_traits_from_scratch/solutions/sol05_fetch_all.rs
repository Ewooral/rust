//! Solution to exercise 5: look many users up at once with `tokio::spawn` (pass 3 tests).
//!
//! Implement both functions. Each must start ALL its lookups before waiting
//! for any of them: one `tokio::spawn` per id, each task owning its own
//! `Arc::clone` of the service. Then await the `JoinHandle`s in the order
//! of `ids`, so the answers come back in that order.
//!
//!   - `fetch_all`: the answers, in order.
//!   - `fetch_all_within`: the same, but each lookup gets at most `limit`
//!     (`tokio::time::timeout(limit, future)` inside the task). A lookup that
//!     runs out of time becomes `Err(TimedOut(id))`.
//!
//! A `for` loop that awaits one lookup before starting the next gives the
//! right answers, but the concurrency test catches it.
//!
//! Test: cargo test -p async_traits_from_scratch --features solutions --bin sol05_fetch_all

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, PartialEq)]
struct TimedOut(u32);

trait UserService: Send + Sync {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

/// Answers every id after `id` milliseconds (so id 500 is slow), and keeps
/// track of how many lookups were running at the same moment.
#[derive(Default)]
struct SlowUsers {
    running: AtomicUsize,
    most_at_once: AtomicUsize,
}

impl UserService for SlowUsers {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move {
            let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
            self.most_at_once.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(u64::from(id))).await;
            self.running.fetch_sub(1, Ordering::SeqCst);
            id.is_multiple_of(2)
                .then(|| User::new(id, format!("user {id}")))
        })
    }
}

/// Looks up every id at the same time; the answers are in the order of `ids`.
async fn fetch_all(service: Arc<dyn UserService>, ids: &[u32]) -> Vec<Option<User>> {
    // Start everything first (`collect` makes the lazy `map` run now)...
    let handles: Vec<_> = ids
        .iter()
        .map(|&id| {
            let service = Arc::clone(&service);
            tokio::spawn(async move { service.get_user(id).await })
        })
        .collect();
    // ...then wait, in order. The tasks are all running meanwhile.
    let mut answers = Vec::with_capacity(handles.len());
    for handle in handles {
        answers.push(handle.await.expect("lookup task panicked"));
    }
    answers
}

/// Like `fetch_all`, but gives each lookup at most `limit`.
async fn fetch_all_within(
    service: Arc<dyn UserService>,
    ids: &[u32],
    limit: Duration,
) -> Vec<Result<Option<User>, TimedOut>> {
    let handles: Vec<_> = ids
        .iter()
        .map(|&id| {
            let service = Arc::clone(&service);
            tokio::spawn(async move {
                tokio::time::timeout(limit, service.get_user(id))
                    .await
                    .map_err(|_elapsed| TimedOut(id))
            })
        })
        .collect();
    let mut answers = Vec::with_capacity(handles.len());
    for handle in handles {
        answers.push(handle.await.expect("lookup task panicked"));
    }
    answers
}

#[tokio::main]
async fn main() {
    let service: Arc<dyn UserService> = Arc::new(SlowUsers::default());
    println!("{:?}", fetch_all(service.clone(), &[2, 3, 4]).await);
    let limit = Duration::from_millis(100);
    println!("{:?}", fetch_all_within(service, &[2, 500], limit).await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn answers_come_back_in_the_order_asked() {
        let service: Arc<dyn UserService> = Arc::new(SlowUsers::default());
        let answers = fetch_all(service, &[30, 3, 20, 10]).await;
        let ids: Vec<Option<u32>> = answers.iter().map(|a| a.as_ref().map(|u| u.id)).collect();
        assert_eq!(ids, [Some(30), None, Some(20), Some(10)]);
        assert!(
            fetch_all(Arc::new(SlowUsers::default()), &[])
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn the_lookups_overlap() {
        let slow = Arc::new(SlowUsers::default());
        fetch_all(slow.clone(), &[20, 20, 20, 20]).await;
        assert_eq!(
            slow.most_at_once.load(Ordering::SeqCst),
            4,
            "all four lookups should be running at the same time"
        );
    }

    #[tokio::test]
    async fn slow_lookups_time_out_and_the_rest_still_answer() {
        let service: Arc<dyn UserService> = Arc::new(SlowUsers::default());
        let answers = fetch_all_within(service, &[2, 500, 3], Duration::from_millis(100)).await;
        assert_eq!(answers.len(), 3);
        assert_eq!(answers[0].as_ref().unwrap().as_ref().unwrap().id, 2);
        assert_eq!(answers[1], Err(TimedOut(500)));
        assert_eq!(answers[2], Ok(None));
    }
}
