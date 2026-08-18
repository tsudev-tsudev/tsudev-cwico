//! UTF-16 conversion helpers.
//!
//! Every `*W` Win32 entry point wants a NUL-terminated UTF-16 buffer, and
//! returns strings as fixed-size arrays that may or may not be terminated.
//! Getting either wrong is how you read past the end of a buffer, so the
//! conversions live in one place with tests.

use windows::core::PCWSTR;

/// Owned NUL-terminated UTF-16 buffer.
///
/// Holding the `Vec` alive is what keeps the pointer valid; `as_pcwstr`
/// borrows from `self`, so the compiler enforces that for us.
pub struct WideString(Vec<u16>);

impl WideString {
    pub fn new(s: &str) -> Self {
        let mut buf: Vec<u16> = s.encode_utf16().collect();
        buf.push(0);
        Self(buf)
    }

    pub fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.0.as_ptr())
    }
}

/// Convert a NUL-terminated (or fully populated) UTF-16 buffer to `String`.
///
/// Stops at the first NUL, and lossily replaces unpaired surrogates rather
/// than failing: a mangled `DisplayName` should still show up in the list.
pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// Convert a raw `*const u16` the API filled in, bounded by `max_len` so a
/// missing terminator cannot run away.
///
/// # Safety
/// `ptr` must be valid for reads of up to `max_len` `u16` values.
pub unsafe fn from_wide_ptr(ptr: *const u16, max_len: usize) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while len < max_len {
        // SAFETY: bounded by max_len, which the caller guarantees is readable.
        if unsafe { *ptr.add(len) } == 0 {
            break;
        }
        len += 1;
    }
    // SAFETY: `len` characters were just proven readable.
    from_wide(unsafe { std::slice::from_raw_parts(ptr, len) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_ascii() {
        let w = WideString::new("Hello");
        assert_eq!(from_wide(&w.0), "Hello");
    }

    #[test]
    fn round_trips_vietnamese() {
        let s = "Gỡ bỏ phần mềm";
        let w = WideString::new(s);
        assert_eq!(from_wide(&w.0), s);
    }

    #[test]
    fn stops_at_the_first_nul() {
        let buf: Vec<u16> = "AB\0CD".encode_utf16().collect();
        assert_eq!(from_wide(&buf), "AB");
    }

    #[test]
    fn handles_a_buffer_with_no_terminator() {
        let buf: Vec<u16> = "ABC".encode_utf16().collect();
        assert_eq!(from_wide(&buf), "ABC");
    }

    #[test]
    fn wide_string_is_nul_terminated() {
        let w = WideString::new("x");
        assert_eq!(w.0.last(), Some(&0));
        assert_eq!(w.0.len(), 2);
    }
}
