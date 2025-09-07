
//! utils/clear_cli.rs
//!
//! Minimal console clear utilities using ANSI escape sequences only.
//! These functions assume an ANSI-capable terminali.
//! # Examples
//!
//! ```
//! use stdt::utils::clear_cli::*;
//!
//! // Clear screen + move cursor to top-left + attempt to clear scrollback.
//! clear().unwrap();
//!
//! // Or write to any `Write` target:
//! let mut buf = Vec::new();
//! write_clear(&mut buf).unwrap();
//! assert_eq!(buf, b"\x1b[H\x1b[2J\x1b[3J");
//! ```

use std::io::{self, Write};

/// ANSI sequence that:
/// - moves the cursor to the home position (`ESC[H`)
/// - clears the visible screen (`ESC[2J`)
/// - attempts to clear the scrollback buffer (`ESC[3J`)
///
/// This constant is private by design; use [`write_clear`] or [`clear`].
const CLEAR_SEQ: &str = "\x1b[H\x1b[2J\x1b[3J";

/// Writes the ANSI clear sequence to the given writer.
/// This does **not** flush automatically. No allocation.
///
/// # Errors
/// Returns any I/O error raised by the underlying writer.
///
/// # Examples
///
/// ```
/// use stdt::utils::clear_cli::write_clear;
/// let mut buf = Vec::new();
/// write_clear(&mut buf).unwrap();
/// assert_eq!(buf, b"\x1b[H\x1b[2J\x1b[3J");
/// ```
#[inline]
pub fn write_clear<W: Write>(mut w: W) -> io::Result<()> {
    w.write_all(CLEAR_SEQ.as_bytes())
}

/// Writes the ANSI clear sequence to `stdout`.
/// This does **not** flush automatically. No allocation.
///
/// # Errors
/// Returns any I/O error raised when writing to `stdout`.
///
/// # Examples
///
/// ```
/// use stdt::utils::clear_cli::clear;
/// clear().unwrap();
/// ```
#[inline]
pub fn clear() -> io::Result<()> {
    write_clear(io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_constant_is_expected() {
        // Ensure we keep using the intended extended clear sequence.
        assert_eq!(CLEAR_SEQ, "\x1b[H\x1b[2J\x1b[3J");
    }

    #[test]
    fn write_clear_writes_correct_bytes() {
        let mut buf = Vec::new();
        write_clear(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[H\x1b[2J\x1b[3J");
    }
}
