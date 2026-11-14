//! The chat server: one Tokio task per connection, a `broadcast` channel to
//! fan messages out to everyone, and a `CancellationToken` plus a
//! `TaskTracker` for graceful shutdown.
//!
//! ```text
//!               +-------------- broadcast::Sender<Event> --------------+
//!               |                                                       |
//!  client A <-> task A --send--> [ channel ] --recv--> task B <-> client B
//!                                    |
//!                                    +----recv--> task C <-> client C
//! ```

use crate::protocol::{Command, parse, validate_name};
use futures::SinkExt; // `send` on a Framed connection
use std::collections::BTreeSet;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::time::{sleep, timeout};
use tokio_stream::StreamExt; // `next` on a Framed connection
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

/// What every client sees when the server stops.
pub const SHUTDOWN_NOTICE: &str = "* server shutting down, bye";

#[derive(Debug, Clone)]
pub struct Config {
    /// How long a new client has to send its name.
    pub name_timeout: Duration,
    /// Longest line accepted, in bytes. Longer lines are refused.
    pub max_line: usize,
    /// How many messages the broadcast channel holds for a slow reader
    /// before that reader starts missing some.
    pub capacity: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name_timeout: Duration::from_secs(30),
            max_line: 1024,
            capacity: 64,
        }
    }
}

/// One message on its way to every connection. `from` lets each task skip
/// its own messages; `Arc<str>` makes cloning it for every receiver cheap.
#[derive(Debug, Clone)]
struct Event {
    from: u64,
    text: Arc<str>,
}

/// State shared by every connection task.
struct Shared {
    // A std Mutex: it's only held for a line of plain code, never across
    // an `.await`.
    names: Mutex<BTreeSet<String>>,
    events: broadcast::Sender<Event>,
    next_id: AtomicU64,
    config: Config,
}

impl Shared {
    fn announce(&self, from: u64, text: String) {
        // `send` fails only when nobody is subscribed: nothing to do then.
        let _ = self.events.send(Event {
            from,
            text: text.into(),
        });
    }

    fn who(&self) -> String {
        let names = self.names.lock().unwrap();
        names.iter().cloned().collect::<Vec<_>>().join(", ")
    }
}

/// Holds a name while its client is connected and frees it when dropped,
/// however the connection ends: a clean `/quit`, a network error, a panic.
struct NameGuard {
    shared: Arc<Shared>,
    name: String,
}

impl Drop for NameGuard {
    fn drop(&mut self) {
        self.shared.names.lock().unwrap().remove(&self.name);
    }
}

type Lines = Framed<TcpStream, LinesCodec>;

/// Accepts clients on `listener` until `shutdown` is cancelled. Then it
/// stops accepting, tells every client, and returns once every connection
/// task has finished.
pub async fn run(
    listener: TcpListener,
    config: Config,
    shutdown: CancellationToken,
) -> io::Result<()> {
    let (events, _) = broadcast::channel(config.capacity);
    let shared = Arc::new(Shared {
        names: Mutex::new(BTreeSet::new()),
        events,
        next_id: AtomicU64::new(1),
        config,
    });
    let tracker = TaskTracker::new();

    loop {
        let (stream, peer) = tokio::select! {
            _ = shutdown.cancelled() => break,
            accepted = listener.accept() => match accepted {
                Ok(pair) => pair,
                Err(e) => {
                    // e.g. too many open files: wait a little, don't spin
                    eprintln!("accept failed: {e}");
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
            },
        };
        let shared = Arc::clone(&shared);
        let shutdown = shutdown.clone();
        tracker.spawn(async move {
            if let Err(e) = handle_client(stream, shared, shutdown).await {
                eprintln!("{peer}: {e}");
            }
        });
    }

    drop(listener); // new connections are refused from here on
    tracker.close(); // no more tasks will be added...
    tracker.wait().await; // ...so this waits for the ones still running
    Ok(())
}

async fn handle_client(
    stream: TcpStream,
    shared: Arc<Shared>,
    shutdown: CancellationToken,
) -> Result<(), LinesCodecError> {
    let mut lines = Framed::new(
        stream,
        LinesCodec::new_with_max_length(shared.config.max_line),
    );
    lines.send("* welcome! what is your name?").await?;

    let Some(guard) = ask_name(&mut lines, &shared, &shutdown).await? else {
        return Ok(());
    };
    let name = guard.name.clone();
    let id = shared.next_id.fetch_add(1, Ordering::Relaxed);

    // Subscribe BEFORE announcing, so nothing sent from now on is missed.
    let mut events = shared.events.subscribe();
    lines
        .send(format!("* hello {name}! online: {}", shared.who()))
        .await?;
    shared.announce(id, format!("* {name} joined"));

    let result = chat(&mut lines, &shared, &shutdown, &mut events, id, &name).await;

    drop(guard); // free the name before telling the others
    shared.announce(id, format!("* {name} left"));
    result
}

/// Asks until the client gives a valid, unused name. `None` means the
/// client left, was too slow, or the server is shutting down.
async fn ask_name(
    lines: &mut Lines,
    shared: &Arc<Shared>,
    shutdown: &CancellationToken,
) -> Result<Option<NameGuard>, LinesCodecError> {
    loop {
        let line = tokio::select! {
            _ = shutdown.cancelled() => {
                lines.send(SHUTDOWN_NOTICE).await?;
                return Ok(None);
            }
            next = timeout(shared.config.name_timeout, lines.next()) => match next {
                Err(_elapsed) => {
                    lines.send("* no name given in time, bye").await?;
                    return Ok(None);
                }
                Ok(None) => return Ok(None), // the client hung up
                Ok(Some(line)) => line?,
            },
        };
        let name = match validate_name(&line) {
            Ok(name) => name.to_string(),
            Err(why) => {
                lines.send(why).await?;
                continue;
            }
        };
        // The lock guard is dropped at the end of this statement, before
        // any `.await`.
        let inserted = shared.names.lock().unwrap().insert(name.clone());
        if inserted {
            return Ok(Some(NameGuard {
                shared: Arc::clone(shared),
                name,
            }));
        }
        lines
            .send(format!("* {name} is taken, try another"))
            .await?;
    }
}

/// The main loop: whichever happens first, a line from this client, a
/// message from another client, or shutdown.
async fn chat(
    lines: &mut Lines,
    shared: &Shared,
    shutdown: &CancellationToken,
    events: &mut broadcast::Receiver<Event>,
    id: u64,
    name: &str,
) -> Result<(), LinesCodecError> {
    loop {
        // All three futures are cancel safe: a half-read line stays in
        // Framed's buffer, and an unreceived message stays in the channel,
        // so losing the race loses nothing.
        tokio::select! {
            _ = shutdown.cancelled() => {
                lines.send(SHUTDOWN_NOTICE).await?;
                return Ok(());
            }
            line = lines.next() => match line {
                None => return Ok(()), // the client hung up
                Some(Err(LinesCodecError::MaxLineLengthExceeded)) => {
                    // After ANY decode error a Framed stream ends (its next
                    // `next()` is `None`), so say why and hang up.
                    let max = shared.config.max_line;
                    lines.send(format!("* line too long (max {max} bytes), bye")).await?;
                    return Ok(());
                }
                Some(Err(e)) => return Err(e),
                Some(Ok(line)) => match parse(&line) {
                    Command::Say(text) => shared.announce(id, format!("{name}: {text}")),
                    Command::Me(action) => shared.announce(id, format!("* {name} {action}")),
                    Command::Who => lines.send(format!("* online: {}", shared.who())).await?,
                    Command::Quit => {
                        lines.send("* bye").await?;
                        return Ok(());
                    }
                    Command::Unknown(command) => {
                        lines
                            .send(format!("* unknown command {command} (try /who, /me, /quit)"))
                            .await?;
                    }
                    Command::Empty => {}
                },
            },
            event = events.recv() => match event {
                Ok(event) if event.from == id => {} // our own message
                Ok(event) => lines.send(&*event.text).await?,
                Err(RecvError::Lagged(missed)) => {
                    lines.send(format!("* you were too slow and missed {missed} messages")).await?;
                }
                Err(RecvError::Closed) => return Ok(()),
            },
        }
    }
}
