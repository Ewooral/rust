//! The server: a `TcpListener` feeding connections to a `ThreadPool`
//! (Book 21.2), with a way to stop it cleanly (Book 21.3, and beyond).

use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::http::{Router, handle_connection};
use crate::pool::ThreadPool;

pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
    router: Arc<Router>,
    stop: Arc<AtomicBool>,
    read_timeout: Duration,
}

impl Server {
    /// Listens on `addr` with a pool of `threads` workers.
    /// Use port 0 (`"127.0.0.1:0"`) to let the OS pick a free port, as the
    /// tests do; [`Server::local_addr`] tells you which one it chose.
    pub fn bind(addr: impl ToSocketAddrs, threads: usize, router: Router) -> io::Result<Server> {
        let pool = ThreadPool::build(threads)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        Ok(Server {
            listener: TcpListener::bind(addr)?,
            pool,
            router: Arc::new(router),
            stop: Arc::new(AtomicBool::new(false)),
            read_timeout: Duration::from_secs(5),
        })
    }

    /// How long a worker waits for a client to send its request before
    /// giving up. Without a limit, clients that connect and say nothing
    /// could occupy every worker forever.
    pub fn with_read_timeout(mut self, timeout: Duration) -> Server {
        self.read_timeout = timeout;
        self
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// A handle another thread can use to stop [`Server::run`].
    pub fn shutdown_handle(&self) -> io::Result<ShutdownHandle> {
        Ok(ShutdownHandle {
            addr: self.local_addr()?,
            stop: Arc::clone(&self.stop),
        })
    }

    /// Accepts connections and hands each one to the pool, until either
    /// `limit` connections have been accepted (`Some(2)` is the Book's
    /// `.take(2)`) or a [`ShutdownHandle`] asks it to stop.
    ///
    /// Consumes the server: when `run` returns, the pool is dropped, which
    /// waits for every request already in progress (graceful shutdown).
    pub fn run(self, limit: Option<usize>) {
        // The Book's `.take(2)`; `usize::MAX` means "no limit" in practice.
        let limit = limit.unwrap_or(usize::MAX);
        for stream in self.listener.incoming().take(limit) {
            if self.stop.load(Ordering::SeqCst) {
                break; // woken up by ShutdownHandle::shutdown
            }
            match stream {
                Ok(stream) => {
                    if let Err(e) = stream.set_read_timeout(Some(self.read_timeout)) {
                        eprintln!("could not set a read timeout: {e}");
                    }
                    let router = Arc::clone(&self.router);
                    self.pool.execute(move || {
                        if let Err(e) = handle_connection(stream, &router) {
                            eprintln!("connection error: {e}");
                        }
                    });
                }
                Err(e) => eprintln!("could not accept a connection: {e}"),
            }
        }
        println!("Shutting down.");
        // `self` (and with it the pool) is dropped here.
    }
}

/// Stops a running [`Server`] from another thread.
#[derive(Debug, Clone)]
pub struct ShutdownHandle {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
}

impl ShutdownHandle {
    /// Asks the server to stop, and wakes it up.
    ///
    /// `incoming()` blocks inside `accept()` until a client connects, so
    /// setting the flag alone isn't enough: the handle connects to the server
    /// itself, `accept` returns, and the loop sees the flag.
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::SeqCst);
        let mut addr = self.addr;
        if addr.ip().is_unspecified() {
            addr.set_ip(Ipv4Addr::LOCALHOST.into()); // can't connect to 0.0.0.0 everywhere
        }
        // If the server already stopped there is nothing to wake: ignore errors.
        let _ = TcpStream::connect_timeout(&addr, Duration::from_secs(1));
    }
}
