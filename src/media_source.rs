//! Safe pure MediaSource (file path holder core).
//!
//! Holds a path and a looping flag. Empty paths rejected.
//! No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaError {
    EmptyPath,
}

#[derive(Clone, Debug)]
pub struct MediaSource {
    path: String,
    looping: bool,
}

impl MediaSource {
    pub fn new(path: &str, looping: bool) -> Result<Self, MediaError> {
        if path.is_empty() {
            return Err(MediaError::EmptyPath);
        }
        Ok(Self {
            path: path.to_string(),
            looping,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn set_path(&mut self, path: &str) -> Result<(), MediaError> {
        if path.is_empty() {
            return Err(MediaError::EmptyPath);
        }
        self.path = path.to_string();
        Ok(())
    }

    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(MediaSource::new("", true).is_err());
    }

    #[test]
    fn differential() {
        let mut m = MediaSource::new("clip.mp4", false).unwrap();
        assert_eq!(m.path(), "clip.mp4");
        assert!(!m.looping());
        m.set_path("loop.mp4").unwrap();
        m.set_looping(true);
        assert_eq!(m.path(), "loop.mp4");
        assert!(m.looping());
    }
}
