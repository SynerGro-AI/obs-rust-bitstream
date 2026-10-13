//! Safe pure FileCache (byte store core).
//!
//! Keeps the bytes of a file so a fingerprint can rebuild it.
//! Empty paths rejected. No unwraps.
//!
//! Differential: store + rebuild round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheError {
    EmptyPath,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct FileCache {
    files: BTreeMap<String, Vec<u8>>,
}

impl FileCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, path: &str, bytes: &[u8]) -> Result<(), CacheError> {
        if path.is_empty() {
            return Err(CacheError::EmptyPath);
        }
        self.files.insert(path.to_string(), bytes.to_vec());
        Ok(())
    }

    /// Rebuild a missing file from the cache. Returns the bytes.
    pub fn rebuild(&self, path: &str) -> Result<&[u8], CacheError> {
        self.files.get(path).map(|v| v.as_slice()).ok_or(CacheError::Missing)
    }

    pub fn contains(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn remove(&mut self, path: &str) -> bool {
        self.files.remove(path).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let c = FileCache::new();
        assert_eq!(c.count(), 0);
    }

    #[test]
    fn empty_path() {
        let mut c = FileCache::new();
        assert!(c.store("", b"x").is_err());
    }

    #[test]
    fn differential() {
        let mut c = FileCache::new();
        c.store("a.png", b"image").unwrap();
        assert_eq!(c.rebuild("a.png").unwrap(), b"image");
        assert!(c.rebuild("b.wav").is_err());
    }
}
