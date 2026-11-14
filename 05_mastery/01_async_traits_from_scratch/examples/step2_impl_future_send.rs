//! Step 2: what `async fn` in a trait desugars to, `-> impl Future`, and
//! adding `+ Send` so callers can `tokio::spawn` the future.
//!
//! Run: cargo run -p async_traits_from_scratch --example step2_impl_future_send

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

/// "Ada (id 1)", or "nobody".
fn label(user: Option<User>) -> String {
    user.map_or("nobody".into(), |u| format!("{} (id {})", u.name, u.id))
}

// Step 1 wrote `async fn get_user(&self, id: u32) -> Option<User>;`. That is
// shorthand for this, minus the `+ Send`: an async fn RETURNS a future.
// With step 1's version, `spawn_lookup` below failed:
//     error: future cannot be sent between threads safely   (no error code)
//     ...the trait `Send` is not implemented for `impl Future<Output = Option<User>>`
// The trait never promised that every impl's future is `Send`, and
// `tokio::spawn` may move a task to another thread. Writing the return type
// out lets us make that promise (the compiler's own help suggests it).
trait UserService {
    fn get_user(&self, id: u32) -> impl Future<Output = Option<User>> + Send;
}

struct Database {
    users: HashMap<u32, User>,
}

impl UserService for Database {
    // An impl may still write `async fn`. The compiler checks that this
    // future really is `Send` (it is: it only holds `&Database` and a `u32`).
    async fn get_user(&self, id: u32) -> Option<User> {
        tokio::task::yield_now().await;
        self.users.get(&id).cloned()
    }
}

struct Guests;

impl UserService for Guests {
    // ...or spell it out: a plain fn returning an `async move` block.
    // (Clippy prefers `async fn`; here the long form is the point.)
    #[allow(clippy::manual_async_fn)]
    fn get_user(&self, id: u32) -> impl Future<Output = Option<User>> + Send {
        async move { Some(User::new(id, format!("guest #{id}"))) }
    }
}

// Generic (static dispatch) again. `S: Send + Sync + 'static` lets the
// `Arc<S>` move into the task; `+ Send` in the trait covers the future.
fn spawn_lookup<S>(service: Arc<S>, id: u32) -> tokio::task::JoinHandle<Option<User>>
where
    S: UserService + Send + Sync + 'static,
{
    tokio::spawn(async move { service.get_user(id).await })
}

#[tokio::main]
async fn main() {
    let database = Arc::new(Database {
        users: HashMap::from([(1, User::new(1, "Ada"))]),
    });

    let a = spawn_lookup(database.clone(), 1);
    let b = spawn_lookup(Arc::new(Guests), 7);
    println!("database: {}", label(a.await.unwrap()));
    println!("guests:   {}", label(b.await.unwrap()));

    // Still not dyn compatible: `impl Future` is still a different hidden
    // type in every impl.
    // let services: Vec<Box<dyn UserService>> = vec![Box::new(Guests)];
    // ^ E0038: the trait `UserService` is not dyn compatible
    //          ...because method `get_user` references an `impl Trait` type in its return type
}
