//! Safe pure PipeSuggest (AI suggestion into pipeline weights).
//!
//! Writes an AI curve or effect suggestion into the
//! pipeline weight keys. No unwraps.
//!
//! Differential: a dark frame writes a lift.

use crate::ai::AiAssistant;
use crate::ai_tools::AiTools;
use crate::live_weights::LiveWeights;
use crate::video_frame::VideoFrame;

pub struct PipeSuggest;

impl PipeSuggest {
    /// Write the AI curve mid into pipe.bright.
    pub fn bright(ai: &AiAssistant, frame: &VideoFrame, weights: &mut LiveWeights) {
        let s = AiTools::suggest_curve(ai, frame);
        let mid = s.points.get(1).map(|p| p.1 as f32).unwrap_or(128.0);
        let lift = (mid - 128.0).clamp(-100.0, 100.0);
        let _ = weights.set("pipe.bright", lift);
    }

    /// Write the AI effect strength into pipe.glow.
    pub fn glow(ai: &AiAssistant, frame: &VideoFrame, weights: &mut LiveWeights) {
        let s = AiTools::suggest_effect(ai, frame);
        let _ = weights.set("pipe.glow", s.strength as f32);
    }

    /// Set a full pipeline from one AI pass.
    /// Bright, glow, and a mild sharpen.
    pub fn full(ai: &AiAssistant, frame: &VideoFrame, weights: &mut LiveWeights) {
        Self::bright(ai, frame, weights);
        Self::glow(ai, frame, weights);
        let avg = frame
            .planes
            .first()
            .map(|p| {
                let sum: u32 = p.iter().map(|v| *v as u32).sum();
                sum / p.len().max(1) as u32
            })
            .unwrap_or(128);
        let sharp = if avg < 100 { 20.0 } else { 0.0 };
        let _ = weights.set("pipe.sharpen", sharp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AiAssistant, AiConfig};
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let ai = AiAssistant::new(AiConfig::default());
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(40);
        let mut w = LiveWeights::new();
        PipeSuggest::bright(&ai, &f, &mut w);
        assert!(w.get("pipe.bright").unwrap() > 0.0);
        PipeSuggest::glow(&ai, &f, &mut w);
        assert!(w.get("pipe.glow").is_ok());
    }
}
