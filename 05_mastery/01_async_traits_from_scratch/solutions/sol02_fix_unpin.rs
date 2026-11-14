//! Solution to exercise 2: make it compile (E0277: "cannot be unpinned").
//!
//! Someone boxed the future so the trait is dyn compatible, but forgot the
//! pin. `.await` polls a future, polling needs `Pin<&mut Self>`, and a
//! `Box<dyn Future>` can only give that if the future inside is `Unpin`,
//! which a `dyn Future` can't promise.
//!
//! Fix the trait and both impls, not `main`: return
//! `Pin<BoxFuture<'_, Option<User>>>`, and build it
//! with `Box::pin` instead of `Box::new`. Bonus: give that type a short name
//! with `type BoxFuture<'a, T> = …;`.
//!
//! Answer: `Box::pin` puts the future on the heap AND promises it will never
//! move again, and a `Pin<Box<dyn Future>>` is a future `.await` can poll.
//!
//! Run: cargo run -p async_traits_from_scratch --features solutions --bin sol02_fix_unpin

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait UserService {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

struct Database {
    users: HashMap<u32, User>,
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            self.users.get(&id).cloned()
        })
    }
}

struct Robots;

impl UserService for Robots {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move { Some(User::new(id, format!("robot-{id:03}"))) })
    }
}

#[tokio::main]
async fn main() {
    let services: Vec<Box<dyn UserService>> = vec![
        Box::new(Database {
            users: HashMap::from([(1, User::new(1, "Grace"))]),
        }),
        Box::new(Robots),
    ];
    for service in &services {
        let user = service.get_user(1).await;
        match user {
            Some(user) => println!("found {} (id {})", user.name, user.id),
            None => println!("not found"),
        }
    }
}
