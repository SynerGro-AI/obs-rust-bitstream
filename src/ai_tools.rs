//! Safe pure AiTools (AI suggestions for photo tools).
//!
//! Uses the existing AiAssistant to suggest settings for
//! curves, blend, color balance, selection, layers, and
//! effects. Returns pure values the tools can apply.
//! No unwraps.
//!
//! Differential: a dark frame gets a lift suggestion.

use crate::ai::AiAssistant;
use crate::blend::BlendMode;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug)]
pub struct CurveSuggestion {
    pub points: Vec<(u8, u8)>,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct BalanceSuggestion {
    pub shadow: i8,
    pub mid: i8,
    pub highlight: i8,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct EffectSuggestion {
    pub name: String,
    pub strength: u8,
    pub reason: String,
}

pub struct AiTools;

impl AiTools {
    /// Suggest a curve based on average luminance.
    pub fn suggest_curve(ai: &AiAssistant, frame: &VideoFrame) -> CurveSuggestion {
        let avg = Self::average(frame);
        let points = if avg < 80 {
            vec![(0, 0), (128, 180), (255, 255)]
        } else if avg > 180 {
            vec![(0, 0), (128, 80), (255, 255)]
        } else {
            vec![(0, 0), (128, 128), (255, 255)]
        };
        let reason = ai.assist(&format!("curve for avg {}", avg));
        CurveSuggestion { points, reason }
    }

    /// Suggest color balance shifts.
    pub fn suggest_balance(ai: &AiAssistant, frame: &VideoFrame) -> BalanceSuggestion {
        let avg = Self::average(frame);
        let (shadow, mid, highlight) = if avg < 80 {
            (10, 15, 5)
        } else if avg > 180 {
            (-5, -10, -5)
        } else {
            (0, 0, 0)
        };
        let reason = ai.assist(&format!("balance for avg {}", avg));
        BalanceSuggestion {
            shadow,
            mid,
            highlight,
            reason,
        }
    }

    /// Suggest a blend mode for two frames.
    pub fn suggest_blend(ai: &AiAssistant, base_avg: u8, over_avg: u8) -> (BlendMode, String) {
        let mode = if over_avg < base_avg {
            BlendMode::Screen
        } else {
            BlendMode::Multiply
        };
        let reason = ai.assist(&format!("blend {} over {}", over_avg, base_avg));
        (mode, reason)
    }

    /// Suggest an effect (glow, shadow, blur, sharpen).
    pub fn suggest_effect(ai: &AiAssistant, frame: &VideoFrame) -> EffectSuggestion {
        let avg = Self::average(frame);
        let (name, strength) = if avg < 60 {
            ("inner_glow", 80)
        } else if avg > 200 {
            ("drop_shadow", 120)
        } else {
            ("sharpen", 64)
        };
        let reason = ai.assist(&format!("effect for avg {}", avg));
        EffectSuggestion {
            name: name.to_string(),
            strength,
            reason,
        }
    }

    /// Suggest a magic-wand tolerance.
    pub fn suggest_tolerance(ai: &AiAssistant, frame: &VideoFrame) -> (u8, String) {
        let avg = Self::average(frame);
        let tol = if avg < 64 || avg > 192 { 20 } else { 10 };
        let reason = ai.assist(&format!("tolerance for avg {}", avg));
        (tol, reason)
    }

    fn average(frame: &VideoFrame) -> u8 {
        if frame.planes.is_empty() || frame.planes[0].is_empty() {
            return 0;
        }
        let sum: u32 = frame.planes[0].iter().map(|p| *p as u32).sum();
        (sum / frame.planes[0].len() as u32) as u8
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
        let mut dark = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        dark.planes[0].fill(40);
        let curve = AiTools::suggest_curve(&ai, &dark);
        assert!(curve.points[1].1 > 128);
        let effect = AiTools::suggest_effect(&ai, &dark);
        assert_eq!(effect.name, "inner_glow");
    }
}
