//! Safe pure Fingerprint (file identity core).
//!
//! Stores a hash and size for each path. Empty paths
//! rejected. No unwraps.
//!
//! Differential: store + lookup round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FingerprintError {
    EmptyPath,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Fingerprint {
    pub hash: u64,
    pub size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct FingerprintStore {
    prints: BTreeMap<String, Fingerprint>,
}

impl FingerprintStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Simple FNV-1a hash of the bytes.
    pub fn hash_of(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub fn store(&mut self, path: &str, bytes: &[u8]) -> Result<(), FingerprintError> {
        if path.is_empty() {
            return Err(FingerprintError::EmptyPath);
        }
        self.prints.insert(
            path.to_string(),
            Fingerprint {
                hash: Self::hash_of(bytes),
                size: bytes.len() as u64,
            },
        );
        Ok(())
    }

    pub fn get(&self, path: &str) -> Option<&Fingerprint> {
        self.prints.get(path)
    }

    /// Paths that are missing but have a fingerprint.
    pub fn rebuildable(&self, missing: &crate::missing_files::MissingFiles) -> Vec<String> {
        missing
            .list()
            .into_iter()
            .filter(|p| self.prints.contains_key(*p))
            .map(|s| s.to_string())
            .collect()
    }

    pub fn count(&self) -> usize {
        self.prints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::missing_files::MissingFiles;

    #[test]
    fn create() {
        let s = FingerprintStore::new();
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn empty_path() {
        let mut s = FingerprintStore::new();
        assert!(s.store("", b"x").is_err());
    }

    #[test]
    fn differential() {
        let mut s = FingerprintStore::new();
        s.store("a.png", b"image").unwrap();
        let fp = s.get("a.png").unwrap();
        assert_eq!(fp.size, 5);
        assert_eq!(fp.hash, FingerprintStore::hash_of(b"image"));

        let mut missing = MissingFiles::new();
        missing.add("a.png").unwrap();
        missing.add("b.wav").unwrap();
        let rebuild = s.rebuildable(&missing);
        assert_eq!(rebuild, vec!["a.png"]);
    }
}
