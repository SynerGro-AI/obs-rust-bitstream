//! Safe pure SequenceReverse (reverse frames core).
//!
//! Returns a new sequence with frames in reverse order.
//! No unwraps.
//!
//! Differential: the last frame becomes the first.

use crate::frame_sequence::FrameSequence;

pub struct SequenceReverse;

impl SequenceReverse {
    /// Reverse the frames.
    pub fn reverse(seq: &FrameSequence) -> FrameSequence {
        let mut out = FrameSequence::new();
        for i in (0..seq.len()).rev() {
            if let Ok(f) = seq.get(i) {
                out.push(f.clone());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;
    use crate::video_frame::VideoFrame;

    #[test]
    fn differential() {
        let mut seq = FrameSequence::new();
        let mut a = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        a.planes[0].fill(10);
        let mut b = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        b.planes[0].fill(20);
        seq.push(a);
        seq.push(b);
        let rev = SequenceReverse::reverse(&seq);
        assert_eq!(rev.get(0).unwrap().planes[0][0], 20);
        assert_eq!(rev.get(1).unwrap().planes[0][0], 10);
    }
}
