//! Building a stack of services at run time, and using it from many tasks.

use std::sync::Arc;
use std::time::Duration;

use crate::backends::{InMemoryUsers, RemoteUsers};
use crate::layers::{CachedUsers, FallbackUsers, TimeoutUsers};
use crate::service::{LookupError, User, UserService};

/// The stacks [`build_stack`] knows, from simplest to most careful.
pub const PRESETS: [&str; 4] = ["memory", "remote", "cached", "resilient"];

/// Builds one of the [`PRESETS`] around the two backends you pass in, or
/// `None` for an unknown name.
///
/// The result's concrete type depends on a string known only at run time,
/// which is exactly why the trait has to be dyn compatible.
///
/// | preset      | stack                                             |
/// |-------------|---------------------------------------------------|
/// | `memory`    | memory                                            |
/// | `remote`    | remote                                            |
/// | `cached`    | cache, then remote                                |
/// | `resilient` | cache, then (remote with a time limit, or memory) |
pub fn build_stack(
    preset: &str,
    remote: Arc<RemoteUsers>,
    memory: Arc<InMemoryUsers>,
    time_limit: Duration,
) -> Option<Arc<dyn UserService>> {
    let stack: Arc<dyn UserService> = match preset {
        "memory" => memory,
        "remote" => remote,
        "cached" => Arc::new(CachedUsers::new(remote)),
        "resilient" => {
            let guarded = Arc::new(TimeoutUsers::new(remote, time_limit));
            let fallback = Arc::new(FallbackUsers::new(guarded, memory));
            Arc::new(CachedUsers::new(fallback))
        }
        _ => return None,
    };
    Some(stack)
}

/// Looks up every id at the same time, one spawned task each, and returns
/// the answers in the same order as `ids`.
///
/// Each task owns a clone of the `Arc`, so it is `'static`, and
/// `UserService: Send + Sync` makes the clone `Send`: the two things
/// `tokio::spawn` asks for.
pub async fn lookup_many(
    service: Arc<dyn UserService>,
    ids: &[u32],
) -> Vec<Result<User, LookupError>> {
    let handles: Vec<_> = ids
        .iter()
        .map(|&id| {
            let service = Arc::clone(&service);
            tokio::spawn(async move { service.get_user(id).await })
        })
        .collect();

    let mut answers = Vec::with_capacity(handles.len());
    for handle in handles {
        answers.push(handle.await.expect("a lookup task panicked"));
    }
    answers
}
