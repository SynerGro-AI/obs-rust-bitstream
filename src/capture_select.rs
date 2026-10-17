//! Safe pure CaptureSelect (pick a source).
//!
//! Picks a capture source. Synthetic is always
//! available. Real cameras are OS-specific.
//! No unwraps.
//!
//! Differential: synthetic produces frames.

use crate::capture::{Capture, SyntheticCapture};
use crate::file_capture::{FileCapture, FileCaptureError};
use crate::video_frame::VideoFrame;

pub enum CaptureSource {
    Synthetic(SyntheticCapture),
    File(FileCapture),
}

impl CaptureSource {
    /// Always-available source for tests.
    pub fn synthetic(width: u32, height: u32) -> Self {
        CaptureSource::Synthetic(SyntheticCapture::new(width, height))
    }

    /// Source from a raw file.
    pub fn file(path: &str, width: u32, height: u32) -> Result<Self, FileCaptureError> {
        Ok(CaptureSource::File(FileCapture::open(path, width, height)?))
    }

    /// Next frame from the source.
    pub fn frame(&mut self) -> Option<VideoFrame> {
        match self {
            CaptureSource::Synthetic(c) => c.frame(),
            CaptureSource::File(c) => c.frame(),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.width(),
            CaptureSource::File(c) => c.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.height(),
            CaptureSource::File(c) => c.height(),
        }
    }

    /// Available source kinds.
    pub fn kinds() -> &'static [&'static str] {
        &["synthetic", "file", "windows", "apple", "linux"]
    }

    /// True if the kind works on this OS.
    pub fn available(kind: &str) -> bool {
        match kind {
            "synthetic" | "file" => true,
            "windows" => cfg!(windows),
            "apple" => cfg!(any(target_os = "macos", target_os = "ios")),
            "linux" => cfg!(target_os = "linux"),
            _ => false,
        }
    }

    /// Sources that work on this OS.
    pub fn available_kinds() -> Vec<&'static str> {
        Self::kinds()
            .iter()
            .copied()
            .filter(|k| Self::available(k))
            .collect()
    }

    /// Best available source for tests. Always synthetic.
    pub fn best() -> Self {
        Self::synthetic(16, 16)
    }

    /// Short summary of this source.
    pub fn summary(&self) -> String {
        match self {
            CaptureSource::Synthetic(_) => format!("synthetic {}x{}", self.width(), self.height()),
            CaptureSource::File(_) => format!("file {}x{}", self.width(), self.height()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut src = CaptureSource::synthetic(8, 8);
        let a = src.frame().unwrap();
        let b = src.frame().unwrap();
        assert_ne!(a.planes[0][0], b.planes[0][0]);
        assert_eq!(src.width(), 8);
        let path = "/tmp/sel_frames.raw";
        std::fs::write(path, &[7u8; 16]).unwrap();
        let mut file_src = CaptureSource::file(path, 4, 4).unwrap();
        let f = file_src.frame().unwrap();
        assert_eq!(f.planes[0][0], 7);
        assert!(CaptureSource::kinds().contains(&"synthetic"));
        assert!(CaptureSource::kinds().contains(&"file"));
        assert!(CaptureSource::available("synthetic"));
        assert!(CaptureSource::available("file"));
        let avail = CaptureSource::available_kinds();
        assert!(avail.contains(&"synthetic"));
        assert!(avail.contains(&"file"));
        let mut best = CaptureSource::best();
        assert!(best.frame().is_some());
        assert!(best.summary().contains("synthetic"));
    }
}
