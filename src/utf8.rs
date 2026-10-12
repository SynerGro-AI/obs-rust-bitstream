//! UTF-8 helpers foundation.
//!
//! Fixes the known bug where truncating a filename at exactly 255 bytes
//! can split a multibyte UTF-8 character (producing invalid UTF-8 that
//! aborts recording downstream).
//!
//! Safe path always returns valid UTF-8.

/// Truncate a byte slice to at most `max` bytes without splitting a UTF-8 character.
/// Backs up over continuation bytes (0b10xxxxxx) so the result is always valid.
pub fn truncate_utf8(input: &[u8], max: usize) -> &[u8] {
    if input.len() <= max {
        return input;
    }
    let mut len = max;
    // Back up while we are on a continuation byte
    while len > 0 && (input[len] & 0xC0) == 0x80 {
        len -= 1;
    }
    &input[..len]
}

/// Convenience for the common 255-byte filename limit.
pub fn truncate_filename(input: &[u8]) -> &[u8] {
    truncate_utf8(input, 255)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_unchanged() {
        let s = b"hello.txt";
        assert_eq!(truncate_filename(s), s);
    }

    #[test]
    fn does_not_split_multibyte() {
        // 3-byte character (e.g. zero-width space U+200B is e2 80 8b)
        let mut data = vec![b'a'; 254];
        data.extend_from_slice(&[0xE2, 0x80, 0x8B]); // would be split at 255
        let truncated = truncate_filename(&data);
        // Must not end inside the character
        assert!(std::str::from_utf8(truncated).is_ok());
        assert!(truncated.len() <= 255);
        assert!(truncated.len() < 255 || !truncated.ends_with(&[0x80, 0x8B]));
    }

    #[test]
    fn empty_and_short() {
        assert_eq!(truncate_filename(b""), b"");
        assert_eq!(truncate_utf8(b"abc", 10), b"abc");
    }
}
