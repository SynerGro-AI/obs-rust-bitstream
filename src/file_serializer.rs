//! Safe pure file serializer (OBS file-serializer.h core).
//!
//! The C version wraps FILE* with the serializer function pointers.
//! The safe path reuses MemSerializer and adds load_from / save_to
//! helpers that use std::fs. Short or missing files return false
//! instead of panicking. No raw FILE*, no unwraps.
//!
//! Differential: save then load round-trips the same bytes.

use std::fs;
use std::path::Path;
use crate::diff::assert_equal;
use crate::serializer::{MemSerializer, Serializer};

#[derive(Clone, Debug, Default)]
pub struct FileSerializer {
    inner: MemSerializer,
}

impl FileSerializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let mut s = Self::new();
        s.inner.write(&data);
        s.inner.pos = 0;
        s
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let data = fs::read(path)?;
        Ok(Self::from_bytes(data))
    }

    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        fs::write(path, self.inner.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    pub fn into_inner(self) -> MemSerializer {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn round_trip_memory() {
        let data = b"hello file serializer".to_vec();
        let fs = FileSerializer::from_bytes(data.clone());
        assert_eq!(fs.as_bytes(), &data[..]);
    }

    #[test]
    fn save_load() {
        let dir = env::temp_dir();
        let path = dir.join("obs-rust-fs-test.bin");
        let data = b"test data".to_vec();
        let fs = FileSerializer::from_bytes(data.clone());
        fs.save_to(&path).unwrap();
        let loaded = FileSerializer::load_from(&path).unwrap();
        assert_eq!(loaded.as_bytes(), &data[..]);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_safe() {
        let r = FileSerializer::load_from("/nonexistent/path/obs-test.bin");
        assert!(r.is_err());
    }

    #[test]
    fn differential() {
        let data = b"abc".to_vec();
        let fs = FileSerializer::from_bytes(data.clone());
        let got = fs.as_bytes().to_vec();
        assert_equal(got, data, "file serializer bytes");
    }
}
