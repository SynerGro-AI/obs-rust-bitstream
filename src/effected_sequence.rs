//! Safe pure EffectedSequence (effect then push core).
//!
//! Applies a SceneEffect to a frame and pushes the
//! result into a FrameSequence. The source is unchanged.
//! Reads LiveWeights. No unwraps.
//!
//! Differential: a bright step lifts the pushed frame.

use crate::frame_sequence::FrameSequence;
use crate::live_weights::LiveWeights;
use crate::scene_effect::SceneEffect;
use crate::video_frame::VideoFrame;

pub struct EffectedSequence;

impl EffectedSequence {
    /// Apply the effect and push into the sequence.
    pub fn push(
        seq: &mut FrameSequence,
        effect: &SceneEffect,
        frame: &VideoFrame,
        weights: &LiveWeights,
    ) {
        let mut out = frame.clone();
        let _ = effect.apply(&mut out, weights);
        seq.push(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut seq = FrameSequence::new();
        let mut effect = SceneEffect::new();
        effect.add("bright");
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(40);
        let mut w = LiveWeights::new();
        w.set("pipe.bright", 20.0).unwrap();
        EffectedSequence::push(&mut seq, &effect, &f, &w);
        assert_eq!(seq.len(), 1);
        assert_eq!(seq.get(0).unwrap().planes[0][0], 60);
        assert_eq!(f.planes[0][0], 40);
    }
}
