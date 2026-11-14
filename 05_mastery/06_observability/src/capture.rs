//! Capturing log output in memory, so tests can assert on it.
//!
//! A `fmt` subscriber writes through a [`MakeWriter`]: something that hands
//! out a fresh writer for every log line. `CapturedLogs` hands out writers
//! that append to one shared buffer, and lets the test read it afterwards.
//!
//! ```
//! use observability::capture::CapturedLogs;
//!
//! let logs = CapturedLogs::default();
//! let subscriber = tracing_subscriber::fmt()
//!     .with_writer(logs.clone())
//!     .with_ansi(false) // no colour codes in the text we compare
//!     .without_time()   // no timestamps: the same text every run
//!     .with_target(false)
//!     .finish();
//! tracing::subscriber::with_default(subscriber, || {
//!     tracing::info!(user = "ama", "logged in");
//! });
//! assert_eq!(logs.contents(), " INFO logged in user=\"ama\"\n");
//! ```

use std::io;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::MakeWriter;

/// A shared, growable buffer that log lines are written into.
#[derive(Debug, Clone, Default)]
pub struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

impl CapturedLogs {
    /// Everything written so far, as text.
    pub fn contents(&self) -> String {
        let bytes = self.0.lock().expect("a test panicked while logging");
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// The lines written so far.
    pub fn lines(&self) -> Vec<String> {
        self.contents().lines().map(str::to_string).collect()
    }
}

impl io::Write for CapturedLogs {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("a test panicked while logging")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for CapturedLogs {
    type Writer = CapturedLogs;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone() // a cheap clone: another handle to the same buffer
    }
}
