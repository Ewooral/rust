//! Exercise 5: the actor pattern: one task OWNS the data, everyone else
//! talks to it through a channel.
//!
//! Instead of sharing a `HashMap` behind a lock, `run_actor` owns it, and a
//! cloneable `KvHandle` sends it `Command`s over an `mpsc` channel. Every
//! command carries a `oneshot::Sender` for its reply. Implement:
//!
//! - `run_actor`: receive commands until the channel closes (every handle
//!   dropped), answer each one, and finally RETURN the map.
//!   `Set` inserts and replies with the OLD value (`HashMap::insert` gives
//!   it to you); `Get` replies with a clone of the value; `Len` with the
//!   number of keys. If a reply can't be sent (the asker gave up), ignore it.
//! - `KvHandle::set`, `get`, `len`: make a oneshot channel, send the
//!   command (with the `oneshot::Sender` inside), then await the reply.
//!   If the actor is gone, `set` and `get` return `None` and `len` 0.
//!
//! Hints: `while let Some(cmd) = rx.recv().await { match cmd { .. } }`,
//! `let (reply, answer) = oneshot::channel();`, `self.tx.send(cmd).await`,
//! `answer.await` gives `Result<T, RecvError>`.
//!
//! Test: cargo test -p tokio_runtime --features exercises --bin ex05_kv_actor

#![allow(unused_variables, unused_imports, dead_code, unused_mut)]

use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

enum Command {
    Set {
        key: String,
        value: String,
        reply: oneshot::Sender<Option<String>>,
    },
    Get {
        key: String,
        reply: oneshot::Sender<Option<String>>,
    },
    Len {
        reply: oneshot::Sender<usize>,
    },
}

#[derive(Clone)]
struct KvHandle {
    tx: mpsc::Sender<Command>,
}

/// Starts the actor. The JoinHandle gives the final map once every
/// `KvHandle` has been dropped.
fn start() -> (KvHandle, JoinHandle<HashMap<String, String>>) {
    let (tx, rx) = mpsc::channel(32);
    let actor = tokio::spawn(run_actor(rx));
    (KvHandle { tx }, actor)
}

async fn run_actor(mut rx: mpsc::Receiver<Command>) -> HashMap<String, String> {
    todo!()
}

impl KvHandle {
    async fn set(&self, key: &str, value: &str) -> Option<String> {
        todo!()
    }

    async fn get(&self, key: &str) -> Option<String> {
        todo!()
    }

    async fn len(&self) -> usize {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let (kv, actor) = start();
    println!(
        "set colour -> old value {:?}",
        kv.set("colour", "blue").await
    );
    println!(
        "set colour -> old value {:?}",
        kv.set("colour", "green").await
    );
    println!("get colour -> {:?}", kv.get("colour").await);
    println!("len -> {}", kv.len().await);
    drop(kv); // the last handle: the actor's channel closes
    println!("the actor finished with {:?}", actor.await.unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn set_returns_the_old_value_and_get_reads() {
        let (kv, _actor) = start();
        assert_eq!(kv.get("a").await, None);
        assert_eq!(kv.set("a", "1").await, None);
        assert_eq!(kv.set("a", "2").await, Some("1".to_string()));
        assert_eq!(kv.get("a").await, Some("2".to_string()));
        assert_eq!(kv.len().await, 1);
    }

    #[tokio::test]
    async fn many_tasks_share_one_actor() {
        let (kv, _actor) = start();
        let mut tasks = tokio::task::JoinSet::new();
        for i in 0..100 {
            let kv = kv.clone();
            tasks.spawn(async move { kv.set(&format!("key{i}"), &i.to_string()).await });
        }
        let old_values = tasks.join_all().await;
        assert!(old_values.iter().all(Option::is_none));
        assert_eq!(kv.len().await, 100);
        assert_eq!(kv.get("key42").await, Some("42".to_string()));
    }

    #[tokio::test]
    async fn the_actor_stops_when_every_handle_is_dropped() {
        let (kv, actor) = start();
        let other = kv.clone();
        kv.set("x", "1").await;
        drop(kv);
        other.set("y", "2").await; // still running: one handle is left
        drop(other);
        let map = tokio::time::timeout(std::time::Duration::from_secs(5), actor)
            .await
            .expect("the actor should stop once every handle is gone")
            .unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["y"], "2");
    }
}
