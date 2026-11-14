//! Status reports: binary for the radio, text for the debug console.
//!
//! - **Binary**: Serde works in `no_std` (turn off its `std` feature), and
//!   postcard writes into a `&mut [u8]` you provide, so no heap is needed.
//!   (Lesson 07 used the same crates on the PC.)
//! - **Text**: `core::fmt::Write` is the `no_std` way to format. [`TextBuf`]
//!   is a fixed-size buffer that implements it, so `write!` works without
//!   a `String`.
//! - With the `alloc` feature, [`encode_vec`] returns a `Vec<u8>` instead.

use core::fmt::{self, Write};

use serde::{Deserialize, Serialize};

/// What the pot reports every hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// Hours since power-on.
    pub uptime_hours: u32,
    /// The last good reading, 0 to 1000.
    pub moisture_permille: u16,
    /// Hundredths of a degree Celsius.
    pub temp_centi: i16,
    /// Is the pump running right now?
    pub pump_on: bool,
    /// How many times the pump has started.
    pub waterings: u16,
    /// How many sensor reads have failed.
    pub sensor_faults: u16,
}

/// The largest a `Status` can be once encoded: postcard writes integers as
/// *varints*, at most 5 bytes for a `u32`, 3 for a `u16` or `i16`, 1 for a
/// `bool`.
pub const MAX_ENCODED_LEN: usize = 5 + 3 + 3 + 1 + 3 + 3;

/// Encodes `status` into `buffer`, returning the part that was used.
///
/// # Errors
///
/// `postcard::Error::SerializeBufferFull` if `buffer` is too small.
pub fn encode<'b>(status: &Status, buffer: &'b mut [u8]) -> Result<&'b mut [u8], postcard::Error> {
    postcard::to_slice(status, buffer)
}

/// Decodes what [`encode`] wrote.
///
/// # Errors
///
/// A postcard error if the bytes are cut short or not a `Status`.
pub fn decode(bytes: &[u8]) -> Result<Status, postcard::Error> {
    postcard::from_bytes(bytes)
}

/// Encodes into a new `Vec`. Only with the `alloc` feature.
///
/// # Errors
///
/// A postcard error (in practice, never for a `Status`).
#[cfg(feature = "alloc")]
pub fn encode_vec(status: &Status) -> Result<alloc::vec::Vec<u8>, postcard::Error> {
    postcard::to_allocvec(status)
}

/// Text in a fixed buffer of `N` bytes. Implements `core::fmt::Write`, so
/// `write!(buf, ...)` works with no allocator.
///
/// # Examples
///
/// ```
/// use core::fmt::Write;
/// use no_std_and_embedded::report::TextBuf;
///
/// let mut text: TextBuf<16> = TextBuf::new();
/// write!(text, "{} °C", 21.5).unwrap();
/// assert_eq!(text.as_str(), "21.5 °C");
/// assert!(write!(text, " and much, much more").is_err()); // full
/// ```
#[derive(Clone)]
pub struct TextBuf<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> TextBuf<N> {
    /// An empty buffer.
    pub const fn new() -> TextBuf<N> {
        TextBuf {
            bytes: [0; N],
            len: 0,
        }
    }

    /// The text written so far.
    pub fn as_str(&self) -> &str {
        // Only whole `&str`s are ever copied in (see write_str), so the
        // bytes are always valid UTF-8.
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Default for TextBuf<N> {
    fn default() -> TextBuf<N> {
        TextBuf::new()
    }
}

impl<const N: usize> Write for TextBuf<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        if end > N {
            // fmt::Error carries no message: "it didn't fit" is all we know.
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl<const N: usize> fmt::Debug for TextBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

/// One line for the debug console, such as
/// `t=12h moisture=43.1% temp=21.50C pump=on waterings=2 faults=0`.
///
/// # Errors
///
/// `fmt::Error` if it doesn't fit in 80 bytes (it always does).
pub fn status_line(status: &Status) -> Result<TextBuf<80>, fmt::Error> {
    let mut line = TextBuf::new();
    let temp = status.temp_centi;
    write!(
        line,
        "t={}h moisture={}.{}% temp={}{}.{:02}C pump={} waterings={} faults={}",
        status.uptime_hours,
        status.moisture_permille / 10,
        status.moisture_permille % 10,
        if temp < 0 { "-" } else { "" },
        temp.unsigned_abs() / 100,
        temp.unsigned_abs() % 100,
        if status.pump_on { "on" } else { "off" },
        status.waterings,
        status.sensor_faults,
    )?;
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: Status = Status {
        uptime_hours: 300,
        moisture_permille: 431,
        temp_centi: -205,
        pump_on: true,
        waterings: 2,
        sensor_faults: 0,
    };

    #[test]
    fn round_trips_through_a_stack_buffer() {
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let used = encode(&STATUS, &mut buffer).unwrap();
        // 300 -> [172, 2]; 431 -> [175, 3]; -205 zigzags to 409 -> [153, 3]
        assert_eq!(used, [172, 2, 175, 3, 153, 3, 1, 2, 0]);
        let len = used.len();
        assert_eq!(decode(&buffer[..len]), Ok(STATUS));
    }

    #[test]
    fn the_worst_case_fits_in_max_encoded_len() {
        let worst = Status {
            uptime_hours: u32::MAX,
            moisture_permille: u16::MAX,
            temp_centi: i16::MIN,
            pump_on: true,
            waterings: u16::MAX,
            sensor_faults: u16::MAX,
        };
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        assert_eq!(encode(&worst, &mut buffer).unwrap().len(), MAX_ENCODED_LEN);
    }

    #[test]
    fn a_small_buffer_is_an_error_not_a_crash() {
        let mut tiny = [0u8; 4];
        assert_eq!(
            encode(&STATUS, &mut tiny),
            Err(postcard::Error::SerializeBufferFull)
        );
        assert_eq!(
            decode(&[172, 2]),
            Err(postcard::Error::DeserializeUnexpectedEnd)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn encode_vec_matches_encode() {
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let used = encode(&STATUS, &mut buffer).unwrap();
        assert_eq!(encode_vec(&STATUS).unwrap(), used);
    }

    #[test]
    fn formats_a_status_line_without_a_string() {
        let line = status_line(&STATUS).unwrap();
        assert_eq!(
            line.as_str(),
            "t=300h moisture=43.1% temp=-2.05C pump=on waterings=2 faults=0"
        );
    }

    #[test]
    fn text_buf_refuses_what_does_not_fit() {
        let mut text: TextBuf<5> = TextBuf::new();
        assert!(write!(text, "abc").is_ok());
        assert!(write!(text, "def").is_err());
        assert_eq!(text.as_str(), "abc", "nothing half-written");
        assert_eq!(format!("{text:?}"), "\"abc\"");
    }
}
