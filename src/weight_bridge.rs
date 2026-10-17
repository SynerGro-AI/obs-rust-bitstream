//! Safe pure WeightBridge (AI writes, tools read).
//!
//! AI suggestions are written into LiveWeights once.
//! Later frames read the weights with no new AI call.
//! No unwraps.
//!
//! Differential: write + read round-trips.

use crate::ai::AiAssistant;
use crate::ai_tools::AiTools;
use crate::live_weights::{LiveWeights, WeightError};
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeError {
    Missing,
}

impl From<WeightError> for BridgeError {
    fn from(_: WeightError) -> Self {
        BridgeError::Missing
    }
}

pub struct WeightBridge;

impl WeightBridge {
    /// Write the AI curve suggestion into weights as "curve.mid".
    pub fn from_curve(ai: &AiAssistant, frame: &VideoFrame, weights: &mut LiveWeights) {
        let s = AiTools::suggest_curve(ai, frame);
        let mid = s.points.get(1).map(|p| p.1 as f32).unwrap_or(128.0);
        let _ = weights.set("curve.mid", mid);
    }

    /// Write the AI balance suggestion into weights.
    pub fn from_balance(ai: &AiAssistant, frame: &VideoFrame, weights: &mut LiveWeights) {
        let s = AiTools::suggest_balance(ai, frame);
        let _ = weights.set("balance.shadow", s.shadow as f32);
        let _ = weights.set("balance.mid", s.mid as f32);
        let _ = weights.set("balance.highlight", s.highlight as f32);
    }

    /// Write the AI effect suggestion into weights.
    pub fn from_effect(ai: &AiAssistant, frame: &VideoFrame, weights: &mut LiveWeights) {
        let s = AiTools::suggest_effect(ai, frame);
        let _ = weights.set(&format!("effect.{}", s.name), s.strength as f32);
    }

    /// Read a weight or return a default. No error on missing.
    pub fn read_or(weights: &LiveWeights, name: &str, default: f32) -> f32 {
        weights.get(name).unwrap_or(default)
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
        WeightBridge::from_curve(&ai, &f, &mut w);
        let mid = WeightBridge::read_or(&w, "curve.mid", 0.0);
        assert!(mid > 128.0);
        WeightBridge::from_effect(&ai, &f, &mut w);
        assert!(w.get("effect.inner_glow").is_ok());
    }
}
