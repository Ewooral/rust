//! Step 3: box the future, so every impl returns the SAME type and the trait
//! becomes dyn compatible. Then `.await` complains: the box isn't pinned.
//!
//! Run: cargo run -p async_traits_from_scratch --example step3_box_the_future

use std::collections::HashMap;
use std::future::Future;

use async_traits_from_scratch::User; // { id, name }: the mini project's user

/// "Ada (id 1)", or "nobody".
fn label(user: Option<User>) -> String {
    user.map_or("nobody".into(), |u| format!("{} (id {})", u.name, u.id))
}

// `Box<dyn Future<…>>` is one type, whatever future is inside: a pointer to
// the heap plus a vtable. (`+ Send` is step 2's promise; `+ '_` says the
// future may borrow `self`, and step 6 explains it.)
trait UserService {
    fn get_user(&self, id: u32) -> Box<dyn Future<Output = Option<User>> + Send + '_>;
}

struct Database {
    users: HashMap<u32, User>,
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> Box<dyn Future<Output = Option<User>> + Send + '_> {
        Box::new(async move {
            tokio::task::yield_now().await;
            self.users.get(&id).cloned()
        })
    }
}

struct Guests;

impl UserService for Guests {
    fn get_user(&self, id: u32) -> Box<dyn Future<Output = Option<User>> + Send + '_> {
        Box::new(async move { Some(User::new(id, format!("guest #{id}"))) })
    }
}

#[tokio::main]
async fn main() {
    // This is what steps 1 and 2 couldn't do: pick services at run time.
    let services: Vec<Box<dyn UserService>> = vec![
        Box::new(Database {
            users: HashMap::from([(1, User::new(1, "Ada"))]),
        }),
        Box::new(Guests),
    ];

    for service in &services {
        // let user = service.get_user(1).await;
        // ^ E0277: `dyn Future<Output = Option<User>> + Send` cannot be unpinned
        //   (`.await` needs a future it can poll, and polling needs a pin)

        // `Box::into_pin` turns the box into a `Pin<Box<…>>`, which can be
        // awaited. Step 4 puts the `Pin` in the trait, so callers don't
        // have to remember this.
        let user = Box::into_pin(service.get_user(1)).await;
        println!("{}", label(user));
    }
}
