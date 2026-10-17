//! Safe pure FrameHold (repeat a frame core).
//!
//! Repeats a frame N times into a sequence.
//! No unwraps.
//!
//! Differential: hold 3 produces 3 identical frames.

use crate::frame_sequence::FrameSequence;
use crate::video_frame::VideoFrame;

pub struct FrameHold;

impl FrameHold {
    /// Push the frame `count` times.
    pub fn hold(seq: &mut FrameSequence, frame: &VideoFrame, count: usize) {
        for _ in 0..count {
            seq.push(frame.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut seq = FrameSequence::new();
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(77);
        FrameHold::hold(&mut seq, &f, 3);
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.get(0).unwrap().planes[0][0], 77);
        assert_eq!(seq.get(2).unwrap().planes[0][0], 77);
    }
}
