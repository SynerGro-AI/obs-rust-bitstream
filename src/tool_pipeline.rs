//! Safe pure ToolPipeline (named effect chain core).
//!
//! Runs a list of named effects on a frame. Reads
//! strength from LiveWeights. Unknown names skipped.
//! No unwraps.
//!
//! Differential: a brightness step lifts a dark frame.

use crate::add_noise::AddNoise;
use crate::box_blur::BoxBlur;
use crate::bright_contrast::BrightContrast;
use crate::color_balance::ColorBalance;
use crate::exposure::Exposure;
use crate::frame_sequence::FrameSequence;
use crate::hue_sat::HueSat;
use crate::inner_glow::InnerGlow;
use crate::invert::Invert;
use crate::levels::Levels;
use crate::live_weights::LiveWeights;
use crate::posterize::Posterize;
use crate::sharpen::Sharpen;
use crate::threshold::Threshold;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipeError {
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct ToolPipeline {
    steps: Vec<String>,
}

impl ToolPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) {
        if !name.is_empty() {
            self.steps.push(name.to_string());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// The steps this pipeline knows how to run.
    pub fn available() -> &'static [&'static str] {
        &[
            "bright",
            "exposure",
            "threshold",
            "invert",
            "posterize",
            "noise",
            "blur",
            "sharpen",
            "glow",
            "levels",
            "hue",
            "balance",
        ]
    }

    /// Run each step. Strength comes from live weights.
    pub fn run(&self, frame: &mut VideoFrame, weights: &LiveWeights) -> Result<(), PipeError> {
        for name in &self.steps {
            let key = format!("pipe.{}", name);
            let strength = weights.get(&key).unwrap_or(0.0).clamp(-100.0, 100.0) as i8;
            match name.as_str() {
                "bright" => {
                    let _ = BrightContrast::apply(frame, strength, 0);
                }
                "exposure" => {
                    let _ = Exposure::apply(frame, strength);
                }
                "threshold" => {
                    let t = weights.get("pipe.threshold.level").unwrap_or(128.0) as u8;
                    let _ = Threshold::apply(frame, t);
                }
                "invert" => {
                    let _ = Invert::apply(frame);
                }
                "posterize" => {
                    let levels = weights.get("pipe.posterize.levels").unwrap_or(4.0) as u8;
                    let _ = Posterize::apply(frame, levels.max(2));
                }
                "noise" => {
                    let amount = weights.get("pipe.noise").unwrap_or(0.0) as u8;
                    let _ = AddNoise::apply(frame, amount);
                }
                "blur" => {
                    let _ = BoxBlur::apply(frame);
                }
                "sharpen" => {
                    let amount = weights.get("pipe.sharpen").unwrap_or(0.0) as u8;
                    let _ = Sharpen::apply(frame, amount);
                }
                "glow" => {
                    let strength = weights.get("pipe.glow").unwrap_or(0.0) as u8;
                    let _ = InnerGlow::apply(frame, strength);
                }
                "levels" => {
                    let in_min = weights.get("pipe.levels.in_min").unwrap_or(0.0) as u8;
                    let in_max = weights.get("pipe.levels.in_max").unwrap_or(255.0) as u8;
                    let out_min = weights.get("pipe.levels.out_min").unwrap_or(0.0) as u8;
                    let out_max = weights.get("pipe.levels.out_max").unwrap_or(255.0) as u8;
                    let _ = Levels::apply(frame, in_min, in_max, out_min, out_max);
                }
                "hue" => {
                    let shift = weights.get("pipe.hue").unwrap_or(0.0).clamp(-100.0, 100.0) as i8;
                    let _ = HueSat::apply(frame, shift);
                }
                "balance" => {
                    let shadow = weights
                        .get("pipe.balance.shadow")
                        .unwrap_or(0.0)
                        .clamp(-100.0, 100.0) as i8;
                    let mid = weights
                        .get("pipe.balance.mid")
                        .unwrap_or(0.0)
                        .clamp(-100.0, 100.0) as i8;
                    let high = weights
                        .get("pipe.balance.highlight")
                        .unwrap_or(0.0)
                        .clamp(-100.0, 100.0) as i8;
                    let _ = ColorBalance::apply(frame, shadow, mid, high);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Run the pipeline on every frame in a sequence.
    pub fn run_sequence(
        &self,
        seq: &mut FrameSequence,
        weights: &LiveWeights,
    ) -> Result<(), PipeError> {
        for frame in seq.frames_mut() {
            self.run(frame, weights)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut pipe = ToolPipeline::new();
        pipe.add("bright");
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(40);
        let mut w = LiveWeights::new();
        w.set("pipe.bright", 20.0).unwrap();
        pipe.run(&mut f, &w).unwrap();
        assert_eq!(f.planes[0][0], 60);
        let mut seq = FrameSequence::new();
        seq.push(f);
        pipe.run_sequence(&mut seq, &w).unwrap();
        assert_eq!(seq.get(0).unwrap().planes[0][0], 80);
    }
}
