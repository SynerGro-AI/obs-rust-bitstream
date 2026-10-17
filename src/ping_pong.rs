//! Safe pure PingPong (forward then reverse core).
//!
//! Returns a sequence that plays forward then backward.
//! The middle frame is not duplicated. No unwraps.
//!
//! Differential: the first and last frames match.

use crate::frame_sequence::FrameSequence;
use crate::sequence_reverse::SequenceReverse;

pub struct PingPong;

impl PingPong {
    /// Play forward, then reverse without duplicating the end.
    pub fn bounce(seq: &FrameSequence) -> FrameSequence {
        let mut out = FrameSequence::new();
        for i in 0..seq.len() {
            if let Ok(f) = seq.get(i) {
                out.push(f.clone());
            }
        }
        let rev = SequenceReverse::reverse(seq);
        for i in 1..rev.len() {
            if let Ok(f) = rev.get(i) {
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
        for v in [10u8, 20, 30] {
            let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
            f.planes[0].fill(v);
            seq.push(f);
        }
        let bounced = PingPong::bounce(&seq);
        assert_eq!(bounced.len(), 5);
        assert_eq!(bounced.get(0).unwrap().planes[0][0], 10);
        assert_eq!(bounced.get(2).unwrap().planes[0][0], 30);
        assert_eq!(bounced.get(4).unwrap().planes[0][0], 10);
    }
}
