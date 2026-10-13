//! Safe pure MissingFiles (missing file list core).
//!
//! Tracks files that could not be found. Empty paths
//! rejected. No unwraps.
//!
//! Differential: add + list + clear round-trips.

use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissingError {
    EmptyPath,
}

#[derive(Clone, Debug, Default)]
pub struct MissingFiles {
    paths: BTreeSet<String>,
}

impl MissingFiles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, path: &str) -> Result<(), MissingError> {
        if path.is_empty() {
            return Err(MissingError::EmptyPath);
        }
        self.paths.insert(path.to_string());
        Ok(())
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.contains(path)
    }

    pub fn list(&self) -> Vec<&str> {
        self.paths.iter().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.paths.len()
    }

    pub fn clear(&mut self) {
        self.paths.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let m = MissingFiles::new();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn empty_path() {
        let mut m = MissingFiles::new();
        assert!(m.add("").is_err());
    }

    #[test]
    fn differential() {
        let mut m = MissingFiles::new();
        m.add("a.png").unwrap();
        m.add("b.wav").unwrap();
        m.add("a.png").unwrap();
        assert_eq!(m.count(), 2);
        assert!(m.contains("b.wav"));
        m.clear();
        assert_eq!(m.count(), 0);
    }
}
