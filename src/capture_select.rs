//! Safe pure CaptureSelect (pick a source).
//!
//! Picks a capture source. Synthetic is always
//! available. Real cameras are OS-specific.
//! No unwraps.
//!
//! Differential: synthetic produces frames.

use crate::capture::{Capture, SyntheticCapture};
use crate::video_frame::VideoFrame;

pub enum CaptureSource {
    Synthetic(SyntheticCapture),
}

impl CaptureSource {
    /// Always-available source for tests.
    pub fn synthetic(width: u32, height: u32) -> Self {
        CaptureSource::Synthetic(SyntheticCapture::new(width, height))
    }

    /// Next frame from the source.
    pub fn frame(&mut self) -> Option<VideoFrame> {
        match self {
            CaptureSource::Synthetic(c) => c.frame(),
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            CaptureSource::Synthetic(c) => c.height(),
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
    }
}
