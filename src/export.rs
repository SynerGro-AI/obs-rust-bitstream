//! Safe pure Export (clip to file core).
//!
//! Writes a saved clip to a file path. Empty paths rejected.
//! No unwraps.
//!
//! Differential: export + path round-trips.

use crate::replay_save::{ReplaySave, SaveFormat};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportError {
    EmptyPath,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Export {
    path: String,
    format: SaveFormat,
    frames: usize,
}

impl Export {
    /// Export a saved clip to a path. Returns the export record.
    pub fn write(save: &ReplaySave, name: &str, path: &str) -> Result<Self, ExportError> {
        if path.is_empty() {
            return Err(ExportError::EmptyPath);
        }
        let clip = save.saved(name).ok_or(ExportError::Missing)?;
        Ok(Self {
            path: path.to_string(),
            format: clip.format,
            frames: clip.frames.len(),
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn format(&self) -> SaveFormat {
        self.format
    }

    pub fn frames(&self) -> usize {
        self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instant_replay::InstantReplay;

    #[test]
    fn empty_path() {
        let s = ReplaySave::new();
        assert!(Export::write(&s, "goal", "").is_err());
    }

    #[test]
    fn differential() {
        let mut s = ReplaySave::new();
        let mut r = InstantReplay::new(3);
        r.push(1);
        r.push(2);
        s.save_as("loop", &r, SaveFormat::Gif).unwrap();
        let e = Export::write(&s, "loop", "loop.gif").unwrap();
        assert_eq!(e.path(), "loop.gif");
        assert_eq!(e.format(), SaveFormat::Gif);
        assert_eq!(e.frames(), 2);
    }
}
