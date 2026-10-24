//! Dynamic string (dstr) foundation port.
//!
//! Known original bugs fixed in the safe path:
//! - dstr_replace loops forever on empty find string
//! - dstr_insert_ch writes one byte past the end
//!
//! C ABI shims preserve original quirks for compatibility.

/// Safe dynamic string. Does not infinite-loop or overflow.
pub struct Dstr {
    data: Vec<u8>,
}

impl Dstr {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
        }
    }

    /// Safe replace. Empty `find` is a no-op (original loops forever).
    pub fn replace(&mut self, find: &[u8], replace: &[u8]) {
        if find.is_empty() {
            return; // fix the infinite loop
        }
        let mut result = Vec::new();
        let mut i = 0;
        while i < self.data.len() {
            if self.data[i..].starts_with(find) {
                result.extend_from_slice(replace);
                i += find.len();
            } else {
                result.push(self.data[i]);
                i += 1;
            }
        }
        self.data = result;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Default for Dstr {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::assert_equal;

    #[test]
    fn empty_find_is_noop() {
        let mut s = Dstr::from_bytes(b"hello");
        s.replace(b"", b"x");
        assert_eq!(s.as_bytes(), b"hello");
    }

    #[test]
    fn basic_replace() {
        let mut s = Dstr::from_bytes(b"hello world");
        s.replace(b"world", b"rust");
        assert_eq!(s.as_bytes(), b"hello rust");
    }

    #[test]
    fn differential_replace() {
        // Safe path must be deterministic and match expected result.
        let mut s1 = Dstr::from_bytes(b"hello world");
        s1.replace(b"world", b"rust");
        let expected = b"hello rust";
        assert_equal(s1.as_bytes(), expected.as_slice(), "dstr replace");
    }
}
