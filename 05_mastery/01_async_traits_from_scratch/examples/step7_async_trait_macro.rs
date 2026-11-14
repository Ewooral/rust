//! Step 7: `#[async_trait]` writes steps 3 to 6 for you. Write `async fn`,
//! get `Pin<Box<dyn Future + Send + 'lifetime>>`.
//!
//! Run: cargo run -p async_traits_from_scratch --example step7_async_trait_macro

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

/// "Ada (id 1)", or "nobody".
fn label(user: Option<User>) -> String {
    user.map_or("nobody".into(), |u| format!("{} (id {})", u.name, u.id))
}

// The macro rewrites each `async fn` in the trait into (roughly):
//
//     fn find_by_name<'life0, 'life1, 'async_trait>(
//         &'life0 self,
//         name: &'life1 str,
//     ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'async_trait>>
//     where
//         'life0: 'async_trait,   // the future may borrow self...
//         'life1: 'async_trait,   // ...and name
//         Self: 'async_trait;
//
// Exactly the boxed, pinned, Send, lifetime-bounded future of steps 3-6.
#[async_trait]
trait UserService: Send + Sync {
    async fn get_user(&self, id: u32) -> Option<User>;
    async fn find_by_name(&self, name: &str) -> Option<User>;
}

struct Database {
    users: HashMap<u32, User>,
}

// The impl needs the attribute too, so it gets the same rewrite. Forget it
// and every method fails with
//     E0195: lifetime parameters or bounds on method `get_user` do not match
//            the trait declaration
#[async_trait]
impl UserService for Database {
    async fn get_user(&self, id: u32) -> Option<User> {
        tokio::task::yield_now().await;
        self.users.get(&id).cloned()
    }

    async fn find_by_name(&self, name: &str) -> Option<User> {
        self.users.values().find(|u| u.name == name).cloned()
    }
}

// `?Send` drops the `+ Send` from the futures, for code that never crosses
// threads. Now the implementations may hold `Rc` across an `.await`.
#[async_trait(?Send)]
trait LocalGreeter {
    async fn greet(&self, name: &str) -> String;
}

struct Greeter {
    greeting: Rc<str>,
}

#[async_trait(?Send)]
impl LocalGreeter for Greeter {
    async fn greet(&self, name: &str) -> String {
        let greeting = Rc::clone(&self.greeting); // an Rc, kept across the await
        tokio::task::yield_now().await;
        format!("{greeting}, {name}!")
    }
}

#[tokio::main]
async fn main() {
    let service: Arc<dyn UserService> = Arc::new(Database {
        users: HashMap::from([(1, User::new(1, "Ada")), (2, User::new(2, "Grace"))]),
    });

    // Everything from steps 3-6 still works: dyn, spawn, borrowed arguments.
    let tasks: Vec<_> = (1..=3)
        .map(|id| {
            let service = Arc::clone(&service);
            tokio::spawn(async move { service.get_user(id).await })
        })
        .collect();
    for task in tasks {
        println!("{}", label(task.await.unwrap()));
    }
    println!("{}", label(service.find_by_name("Grace").await));

    let local: Box<dyn LocalGreeter> = Box::new(Greeter {
        greeting: Rc::from("Hello"),
    });
    println!("{}", local.greet("Ada").await);
    // tokio::spawn(async move { local.greet("Ada").await });
    // ^ error: future cannot be sent between threads safely (no error code):
    //   with `?Send` the futures aren't `Send`, so they can't be spawned.
}
