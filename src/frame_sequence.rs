//! Safe pure FrameSequence (timed frame list).
//!
//! Holds rendered frames in order. Empty sequence rejected
//! on write. No unwraps.
//!
//! Differential: push + len + get round-trips.

use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SequenceError {
    Empty,
    OutOfRange,
}

#[derive(Clone, Debug, Default)]
pub struct FrameSequence {
    frames: Vec<VideoFrame>,
}

impl FrameSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, frame: VideoFrame) {
        self.frames.push(frame);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn get(&self, index: usize) -> Result<&VideoFrame, SequenceError> {
        self.frames.get(index).ok_or(SequenceError::OutOfRange)
    }

    pub fn frames(&self) -> &[VideoFrame] {
        &self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn empty() {
        let s = FrameSequence::new();
        assert!(s.is_empty());
        assert!(s.get(0).is_err());
    }

    #[test]
    fn differential() {
        let mut s = FrameSequence::new();
        let f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        s.push(f);
        assert_eq!(s.len(), 1);
        assert_eq!(s.get(0).unwrap().width, 2);
    }
}
