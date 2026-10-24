//! Safe pure PosterizeTime (frame hold core).
//!
//! Holds a frame for N steps. No unwraps.
//!
//! Differential: hold returns the stored frame.

use crate::live_weights::LiveWeights;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug)]
pub struct PosterizeTime {
    held: Option<VideoFrame>,
    every: u32,
    count: u32,
}

impl PosterizeTime {
    pub fn new(every: u32) -> Self {
        Self {
            held: None,
            every: every.max(1),
            count: 0,
        }
    }

    /// Feed a frame. Returns the held frame.
    pub fn step(&mut self, frame: &VideoFrame) -> VideoFrame {
        if self.held.is_none() || self.count.is_multiple_of(self.every) {
            self.held = Some(frame.clone());
        }
        self.count += 1;
        self.held.clone().unwrap_or_else(|| frame.clone())
    }

    /// Set the hold interval from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(v) = weights.get("time.posterize") {
            self.every = (v as u32).max(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut pt = PosterizeTime::new(2);
        let mut a = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        a.planes[0].fill(100);
        let mut b = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        b.planes[0].fill(200);
        let first = pt.step(&a);
        assert_eq!(first.planes[0][0], 100);
        let second = pt.step(&b);
        assert_eq!(second.planes[0][0], 100);
    }
}
