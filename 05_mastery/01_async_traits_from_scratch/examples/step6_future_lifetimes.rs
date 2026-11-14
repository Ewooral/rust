//! Step 6: lifetime bounds on the returned future: `'_`, a named `'a`, and
//! `'static` futures that can be spawned directly.
//!
//! Run: cargo run -p async_traits_from_scratch --example step6_future_lifetimes

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

/// "Ada (id 1)", or "nobody".
fn label(user: Option<User>) -> String {
    user.map_or("nobody".into(), |u| format!("{} (id {})", u.name, u.id))
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait UserService: Send + Sync {
    // 1. `'_`: the future may borrow `self`, so it can't outlive `self`.
    //    Leave it out and `Box<dyn Future …>` means `Box<dyn Future … + 'static>`
    //    (the default for a trait object in a Box), and the impl fails:
    //        error: lifetime may not live long enough   (no error code)
    //        returning this value requires that `'1` must outlive `'static`
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;

    // 2. Two borrowed inputs: name the lifetime, and tie BOTH to the future.
    //    With `'_` the elision rules pick `self`'s lifetime only, and the impl
    //    fails ("lifetime may not live long enough": the method "was supposed
    //    to return data with lifetime `'2` but it is returning data with
    //    lifetime `'1`").
    fn find_by_name<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Option<User>>;

    // 3. `'static`: the future borrows nothing, so it can be spawned as it
    //    is. The impl must clone what it needs BEFORE the `async move`.
    fn get_user_owned(&self, id: u32) -> BoxFuture<'static, Option<User>>;
}

struct Database {
    // Behind an Arc, so `get_user_owned` can give its future a clone.
    users: Arc<HashMap<u32, User>>,
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>> {
        Box::pin(async move { self.users.get(&id).cloned() })
    }

    fn find_by_name<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Option<User>> {
        Box::pin(async move { self.users.values().find(|u| u.name == name).cloned() })
    }

    fn get_user_owned(&self, id: u32) -> BoxFuture<'static, Option<User>> {
        let users = Arc::clone(&self.users); // the future owns this, not `&self`
        Box::pin(async move { users.get(&id).cloned() })
    }
}

#[tokio::main]
async fn main() {
    let database = Database {
        users: Arc::new(HashMap::from([
            (1, User::new(1, "Ada")),
            (2, User::new(2, "Grace")),
        ])),
    };

    println!(
        "get_user(1):          {}",
        label(database.get_user(1).await)
    );
    let wanted = String::from("Grace");
    println!(
        "find_by_name(Grace):  {}",
        label(database.find_by_name(&wanted).await)
    );

    // `tokio::spawn` needs a `'static` future, because the task may run
    // after this function returns. A future that borrows `&database` isn't:
    // let task = tokio::spawn(database.get_user(1));
    // ^ E0597: `database` does not live long enough
    //   (in a function that takes `service: &dyn UserService` the same
    //    mistake is E0521: borrowed data escapes outside of function)
    let task = tokio::spawn(database.get_user_owned(2));
    println!("spawned owned future: {}", label(task.await.unwrap()));

    // The usual fix in real code: share the service in an Arc and move a
    // clone into the task, as step 5 did.
    let shared: Arc<dyn UserService> = Arc::new(database);
    let service = Arc::clone(&shared);
    let task = tokio::spawn(async move { service.get_user(1).await });
    println!("spawned with an Arc:  {}", label(task.await.unwrap()));
}
