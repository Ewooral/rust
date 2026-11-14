//! Step 4: `Pin<Box<dyn Future …>>`, the shape every dyn-compatible async
//! trait uses, and WHY futures need pinning at all.
//!
//! Run: cargo run -p async_traits_from_scratch --example step4_pin_the_box

use std::collections::HashMap;
use std::future::Future;
use std::pin::{Pin, pin};

use async_traits_from_scratch::User; // { id, name }: the mini project's user

/// "Ada (id 1)", or "nobody".
fn label(user: Option<User>) -> String {
    user.map_or("nobody".into(), |u| format!("{} (id {})", u.name, u.id))
}

/// The type every method returns, with a short name. The `futures` crate
/// has the same alias: `futures::future::BoxFuture`.
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait UserService {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

struct Database {
    users: HashMap<u32, User>,
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        // `Box::pin` = put the future on the heap AND promise it never moves.
        Box::pin(async move {
            tokio::task::yield_now().await;
            self.users.get(&id).cloned()
        })
    }
}

/// Compiles only for types that don't mind being moved after a poll.
fn require_unpin<T: Unpin>(_: &T) {}

#[tokio::main]
async fn main() {
    let services: Vec<Box<dyn UserService>> = vec![Box::new(Database {
        users: HashMap::from([(1, User::new(1, "Ada"))]),
    })];
    for service in &services {
        println!("{}", label(service.get_user(1).await)); // no Box::into_pin needed now
    }

    // WHY pin? This future borrows from ITSELF. Its state machine stores
    // `name` and `first_word`, which points into `name`, across the await.
    // If the future were moved to a new address after being polled,
    // `first_word` would point at the OLD address.
    let self_referential = async {
        let name = String::from("Ada Lovelace");
        let first_word = name.split(' ').next().unwrap(); // a borrow of `name`
        tokio::task::yield_now().await; // ...kept alive across this await
        first_word.len()
    };
    // require_unpin(&self_referential);
    // ^ E0277: `{async block@…}` cannot be unpinned
    //   The compiler can't prove this future is safe to move, so it is
    //   `!Unpin`, and polling it needs a `Pin` that promises it won't move.

    // Two ways to pin. On the heap (it can then be stored and returned):
    let mut on_heap: Pin<Box<_>> = Box::pin(self_referential);
    println!("heap-pinned future returned {}", on_heap.as_mut().await);

    // On the stack, with `std::pin::pin!` (only usable inside this scope):
    let on_stack = pin!(async { 40 + 2 });
    println!("stack-pinned future returned {}", on_stack.await);

    // Most ordinary types ARE `Unpin`: moving them is always fine, so `Pin`
    // changes nothing for them. A `Box` is `Unpin` too: moving the box moves
    // the pointer, not the thing on the heap.
    require_unpin(&42);
    require_unpin(&String::from("hi"));
    require_unpin(&std::future::ready(1));
    require_unpin(&on_heap);
    println!("i32, String, Ready<i32> and Pin<Box<…>> are all Unpin");
}
