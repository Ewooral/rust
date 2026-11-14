//! Step 5: `Send + Sync` supertraits, so ONE service can be shared by many
//! `tokio::spawn`ed tasks as an `Arc<dyn UserService>`.
//!
//! Run: cargo run -p async_traits_from_scratch --example step5_send_sync_spawn

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use async_traits_from_scratch::User; // { id, name }: the mini project's user

/// "Ada (id 1)", or "nobody".
fn label(user: Option<User>) -> String {
    user.map_or("nobody".into(), |u| format!("{} (id {})", u.name, u.id))
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Without `: Send + Sync`, the spawn in `main` fails twice:
//     error: future cannot be sent between threads safely   (no error code)
//     = help: the trait `Sync` is not implemented for `dyn UserService`
//     = help: the trait `Send` is not implemented for `dyn UserService`
// A `dyn UserService` could be ANY type, so the compiler only knows what
// the trait promises. Supertraits make every implementor promise it.
//   Send: safe to MOVE to another thread (the Arc in each task needs it)
//   Sync: safe for several threads to use through `&` at the same time
trait UserService: Send + Sync {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

struct Database {
    users: HashMap<u32, User>,
    // Counts lookups through `&self`. A `Cell<u32>` would not do here, for
    // two reasons the compiler reports separately:
    // - the supertrait: `impl UserService for Database` needs `Database: Sync`
    //     E0277: `Cell<u32>` cannot be shared between threads safely
    // - the future borrows `&self`, and a `&Database` is only `Send` if
    //   `Database` is `Sync`:
    //     error: future cannot be sent between threads safely   (no error code)
    //     note: captured value is not `Send` because `&` references cannot be
    //           sent unless their referent is `Sync`
    lookups: AtomicU32,
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move {
            self.lookups.fetch_add(1, Ordering::Relaxed);
            tokio::task::yield_now().await;
            self.users.get(&id).cloned()
        })
    }
}

#[tokio::main]
async fn main() {
    let database = Arc::new(Database {
        users: HashMap::from([(1, User::new(1, "Ada")), (2, User::new(2, "Grace"))]),
        lookups: AtomicU32::new(0),
    });
    // The same Arc, seen through the trait: an unsized coercion.
    let service: Arc<dyn UserService> = database.clone();

    let mut tasks = Vec::new();
    for id in [1, 2, 3] {
        // Each task gets its OWN Arc (a cheap pointer copy), so the task owns
        // everything it uses: `'static`. Moving it to another thread needs
        // `Arc<dyn UserService>: Send`, which needs `dyn UserService: Send + Sync`.
        let service = Arc::clone(&service);
        tasks.push(tokio::spawn(async move {
            let user = service.get_user(id).await;
            (id, user)
        }));
    }
    for task in tasks {
        let (id, user) = task.await.unwrap();
        println!("task for id {id} found {}", label(user));
    }
    println!(
        "lookups counted: {}",
        database.lookups.load(Ordering::Relaxed)
    );
}
