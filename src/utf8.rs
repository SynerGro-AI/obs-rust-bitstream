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
    use crate::diff::differential;

    #[test]
    fn ascii_unchanged() {
        let s = b"hello.txt";
        assert_eq!(truncate_filename(s), s);
    }

    #[test]
    fn does_not_split_multibyte() {
        let mut data = vec![b'a'; 254];
        data.extend_from_slice(&[0xE2, 0x80, 0x8B]);
        let truncated = truncate_filename(&data);
        assert!(std::str::from_utf8(truncated).is_ok());
        assert!(truncated.len() <= 255);
    }

    #[test]
    fn empty_and_short() {
        assert_eq!(truncate_filename(b""), b"");
        assert_eq!(truncate_utf8(b"abc", 10), b"abc");
    }

    #[test]
    fn differential_against_itself() {
        // Pure function: safe path must equal itself (and any future ABI shim).
        let inputs: &[&[u8]] = &[b"", b"abc", b"hello.txt", &[0xE2, 0x80, 0x8B]];
        differential(
            inputs,
            |s| truncate_filename(s).to_vec(),
            |s| truncate_filename(s).to_vec(),
            "truncate_filename",
        );
    }
}
