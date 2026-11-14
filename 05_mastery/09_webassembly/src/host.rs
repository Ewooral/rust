//! What the core needs from the outside world, as a trait.
//!
//! The Game of Life core needs two things it can't do by itself: log a
//! message and get a random number. In a browser those are JavaScript's
//! `console.log` and `Math.random`; on the PC, `println!` and a small
//! random number generator; in tests, a fixed script.
//!
//! Hiding them behind a trait means the core never calls JavaScript
//! directly, so it runs and is tested anywhere. Only ONE line picks the
//! real host, with `#[cfg(target_arch = "wasm32")]`: [`DefaultHost`].

/// The services the core asks for.
pub trait Host {
    /// Shows a message to a developer.
    fn log(&mut self, message: &str);
    /// A random number in `0.0..1.0`.
    fn random(&mut self) -> f64;
}

/// The browser: calls the JavaScript functions imported in
/// [`crate::bindings`]. It compiles everywhere, but calling it outside
/// WebAssembly panics: "cannot call wasm-bindgen imported functions on
/// non-wasm targets".
#[derive(Debug, Default)]
pub struct JsHost;

impl Host for JsHost {
    fn log(&mut self, message: &str) {
        crate::bindings::console_log(message);
    }

    fn random(&mut self) -> f64 {
        crate::bindings::math_random()
    }
}

/// The PC: prints, and makes random numbers with xorshift64* (fine for a
/// game, not for passwords).
#[derive(Debug)]
pub struct StdHost {
    state: u64,
}

impl StdHost {
    /// The same seed gives the same "random" numbers every run.
    pub fn with_seed(seed: u64) -> StdHost {
        StdHost {
            state: seed.max(1), // xorshift must not start at 0
        }
    }
}

impl Default for StdHost {
    fn default() -> StdHost {
        StdHost::with_seed(0x2545_F491_4F6C_DD1D)
    }
}

impl Host for StdHost {
    fn log(&mut self, message: &str) {
        println!("[log] {message}");
    }

    fn random(&mut self) -> f64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let bits = self.state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11; // 53 bits
        bits as f64 / (1u64 << 53) as f64
    }
}

/// The host this build uses: JavaScript in WebAssembly, std elsewhere.
#[cfg(target_arch = "wasm32")]
pub type DefaultHost = JsHost;
/// The host this build uses: JavaScript in WebAssembly, std elsewhere.
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultHost = StdHost;

/// A host for tests: plays back fixed numbers and remembers the logs.
#[derive(Debug, Default)]
pub struct ScriptedHost {
    numbers: Vec<f64>,
    next: usize,
    /// Every message logged so far.
    pub logs: Vec<String>,
}

impl ScriptedHost {
    /// Returns `numbers` in order, then starts again.
    pub fn new(numbers: &[f64]) -> ScriptedHost {
        ScriptedHost {
            numbers: numbers.to_vec(),
            next: 0,
            logs: Vec::new(),
        }
    }
}

impl Host for ScriptedHost {
    fn log(&mut self, message: &str) {
        self.logs.push(message.to_string());
    }

    fn random(&mut self) -> f64 {
        let value = self
            .numbers
            .get(self.next % self.numbers.len().max(1))
            .copied()
            .unwrap_or(0.0);
        self.next += 1;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_host_is_repeatable_and_in_range() {
        let mut a = StdHost::with_seed(42);
        let mut b = StdHost::with_seed(42);
        for _ in 0..1000 {
            let x = a.random();
            assert_eq!(x, b.random());
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn std_host_numbers_look_spread_out() {
        let mut host = StdHost::default();
        let below_half = (0..10_000).filter(|_| host.random() < 0.5).count();
        assert!((4_800..5_200).contains(&below_half), "{below_half}");
    }

    #[test]
    fn scripted_host_repeats_its_script_and_keeps_logs() {
        let mut host = ScriptedHost::new(&[0.25, 0.75]);
        assert_eq!(
            [host.random(), host.random(), host.random()],
            [0.25, 0.75, 0.25]
        );
        host.log("hello");
        assert_eq!(host.logs, ["hello"]);
    }

    #[test]
    fn calling_javascript_on_the_pc_panics() {
        let result = std::panic::catch_unwind(|| JsHost.random());
        assert!(result.is_err());
    }
}
