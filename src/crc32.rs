//! CRC32 foundation.
//!
//! Simple, correct, tested. Ready for C ABI shims later.
//! Uses the standard polynomial 0xEDB88320 (reflected).

const POLY: u32 = 0xEDB88320;

fn table() -> [u32; 256] {
    let mut t = [0u32; 256];
    for i in 0..256 {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { POLY ^ (c >> 1) } else { c >> 1 };
        }
        t[i] = c;
    }
    t
}

/// Compute CRC32 of a byte slice (standard IEEE polynomial, init 0xFFFFFFFF, final XOR).
pub fn crc32(data: &[u8]) -> u32 {
    let table = table();
    let mut crc = 0xFFFFFFFFu32;
    for &b in data {
        let idx = ((crc ^ u32::from(b)) & 0xFF) as usize;
        crc = table[idx] ^ (crc >> 8);
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::differential;

    #[test]
    fn empty() {
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn differential_against_itself() {
        let inputs: &[&[u8]] = &[b"", b"a", b"123456789", b"hello"];
        differential(
            inputs,
            |s| crc32(s),
            |s| crc32(s),
            "crc32",
        );
    }
}
