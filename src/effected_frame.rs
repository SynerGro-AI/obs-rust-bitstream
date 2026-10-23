//! Safe pure EffectedFrame (apply effect then copy core).
//!
//! Runs a SceneEffect on a frame and returns the result.
//! The original is unchanged. Reads LiveWeights. No unwraps.
//!
//! Differential: a bright step lifts the copy, original stays.

use crate::live_weights::LiveWeights;
use crate::scene_effect::SceneEffect;
use crate::video_frame::VideoFrame;

pub struct EffectedFrame;

impl EffectedFrame {
    /// Apply the effect and return a new frame.
    pub fn apply(effect: &SceneEffect, frame: &VideoFrame, weights: &LiveWeights) -> VideoFrame {
        let mut out = frame.clone();
        let _ = effect.apply(&mut out, weights);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut effect = SceneEffect::new();
        effect.add("bright");
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(40);
        let mut w = LiveWeights::new();
        w.set("pipe.bright", 20.0).unwrap();
        let out = EffectedFrame::apply(&effect, &f, &w);
        assert_eq!(out.planes[0][0], 60);
        assert_eq!(f.planes[0][0], 40);
    }
}
