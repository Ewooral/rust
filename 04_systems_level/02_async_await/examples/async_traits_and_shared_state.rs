//! Beyond the Book: `async fn` in traits, why `dyn` needs boxing (by hand
//! or with the `async_trait` crate), and sharing an
//! `Arc<dyn Trait + Send + Sync>` across `tokio::spawn`.
//!
//! Run: cargo run -p async_await --example async_traits_and_shared_state

use async_trait::async_trait;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

// 1. Native `async fn` in a trait (stable since Rust 1.75). Works great
// with generics (static dispatch):
trait Weather {
    async fn temperature(&self, city: &str) -> Option<i32>;
}

struct FakeWeather;

impl Weather for FakeWeather {
    async fn temperature(&self, city: &str) -> Option<i32> {
        sleep(Duration::from_millis(5)).await;
        match city {
            "Accra" => Some(31),
            "London" => Some(12),
            _ => None,
        }
    }
}

async fn report(service: &impl Weather, city: &str) -> String {
    match service.temperature(city).await {
        Some(t) => format!("{city}: {t}°C"),
        None => format!("{city}: unknown"),
    }
}

// ...but NOT as a trait object. Each implementation's future has a
// different hidden type and size, so there's no single vtable entry:
//     let service: Box<dyn Weather> = Box::new(FakeWeather);
//     // E0038: the trait `Weather` is not dyn compatible
//     //        ...because method `temperature` is `async`

// 2. Fix A, by hand: return a boxed future. One concrete return type
// (`Pin<Box<dyn Future + Send + 'a>>`), so the trait is dyn compatible.
trait Store: Send + Sync {
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Option<String>>;
    fn put<'a>(&'a self, key: &'a str, value: String) -> BoxFuture<'a, ()>;
}

#[derive(Default)]
struct MemoryStore {
    // std's Mutex is fine in async code as long as the guard is never held
    // across an `.await`. (If it must be, use `tokio::sync::Mutex`.)
    data: Mutex<HashMap<String, String>>,
}

impl Store for MemoryStore {
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Option<String>> {
        Box::pin(async move {
            sleep(Duration::from_millis(2)).await; // pretend: a network hop
            self.data.lock().unwrap().get(key).cloned()
        })
    }

    fn put<'a>(&'a self, key: &'a str, value: String) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            sleep(Duration::from_millis(2)).await;
            self.data.lock().unwrap().insert(key.to_string(), value);
        })
    }
}

// 3. Fix B: the `async_trait` macro writes exactly that boxing for you, so
// you can keep writing `async fn`.
#[async_trait]
trait Greeter: Send + Sync {
    async fn greet(&self, name: &str) -> String;
}

struct English;
struct Twi;

#[async_trait]
impl Greeter for English {
    async fn greet(&self, name: &str) -> String {
        format!("Hello, {name}")
    }
}

#[async_trait]
impl Greeter for Twi {
    async fn greet(&self, name: &str) -> String {
        sleep(Duration::from_millis(3)).await;
        format!("Akwaaba, {name}")
    }
}

#[tokio::main]
async fn main() {
    println!("1. native async fn in a trait, static dispatch:");
    for city in ["Accra", "London", "Atlantis"] {
        println!("   {}", report(&FakeWeather, city).await);
    }

    // 4. Sharing ONE store between many spawned tasks. `tokio::spawn` needs
    // `Send + 'static` futures, so each task gets its own `Arc` clone, and
    // the trait object must be `Send + Sync` (the `Store: Send + Sync`
    // supertrait guarantees that). With a plain `Arc<dyn SomeTrait>` that
    // isn't Send + Sync, `tokio::spawn` refuses:
    //     error: future cannot be sent between threads safely
    //     ... has type `Arc<dyn SomeTrait>` which is not `Send`
    println!("2. Arc<dyn Store> shared by 5 spawned tasks:");
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    let mut tasks = Vec::new();
    for id in 1..=5 {
        let store = Arc::clone(&store);
        tasks.push(tokio::spawn(async move {
            let key = format!("user:{id}");
            store.put(&key, format!("player {id}")).await;
            store.get(&key).await
        }));
    }
    for task in tasks {
        println!("   a task read back {:?}", task.await.unwrap());
    }
    println!("   one more lookup: {:?}", store.get("user:3").await);

    println!("3. #[async_trait] trait objects, mixed in one Vec:");
    let greeters: Vec<Arc<dyn Greeter>> = vec![Arc::new(English), Arc::new(Twi)];
    let handles: Vec<_> = greeters
        .iter()
        .map(|g| {
            let g = Arc::clone(g);
            tokio::spawn(async move { g.greet("Kofi").await })
        })
        .collect();
    for handle in handles {
        println!("   {}", handle.await.unwrap());
    }
}
