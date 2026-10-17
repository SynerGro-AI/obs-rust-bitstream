//! Safe pure WeightedEffect (effect that reads live weights).
//!
//! Applies an effect using the strength stored in LiveWeights.
//! Missing weight is an error. No unwraps.
//!
//! Differential: a weight of 50 lifts an edge pixel.

use crate::add_noise::AddNoise;
use crate::bevel::Bevel;
use crate::bright_shadow::BrightShadow;
use crate::drop_shadow::DropShadow;
use crate::emboss::Emboss;
use crate::exposure::Exposure;
use crate::gauss_blur::GaussBlur;
use crate::hue_sat::HueSat;
use crate::inner_glow::InnerGlow;
use crate::live_weights::{LiveWeights, WeightError};
use crate::motion_blur::MotionBlur;
use crate::outer_glow::OuterGlow;
use crate::sharpen::Sharpen;
use crate::gradient::Gradient;
use crate::pattern::Pattern;
use crate::stroke::Stroke;
use crate::threshold::Threshold;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WeightedError {
    Missing,
    BadWeight,
}

impl From<WeightError> for WeightedError {
    fn from(_: WeightError) -> Self {
        WeightedError::Missing
    }
}

pub struct WeightedEffect;

impl WeightedEffect {
    fn strength(weights: &LiveWeights, name: &str) -> Result<u8, WeightedError> {
        let v = weights.get(name)?;
        if !(0.0..=255.0).contains(&v) {
            return Err(WeightedError::BadWeight);
        }
        Ok(v as u8)
    }

    /// Apply inner glow using the weight named `name`.
    pub fn glow(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        InnerGlow::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply outer glow using the weight named `name`.
    pub fn outer_glow(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        OuterGlow::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply drop shadow using the weight named `name`.
    pub fn shadow(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        DropShadow::apply(frame, 2, 2, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply bright shadow using the weight named `name`.
    pub fn bright(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        BrightShadow::apply(frame, -1, -1, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply sharpen using the weight named `name`.
    pub fn sharpen(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        Sharpen::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply hue/sat shift. Weight is treated as signed via i8.
    pub fn hue_sat(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        let v = weights.get(name)?;
        if !(-100.0..=100.0).contains(&v) {
            return Err(WeightedError::BadWeight);
        }
        HueSat::apply(frame, v as i8).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply exposure. Weight is stops as i8.
    pub fn exposure(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        let v = weights.get(name)?;
        if !(-8.0..=8.0).contains(&v) {
            return Err(WeightedError::BadWeight);
        }
        Exposure::apply(frame, v as i8).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply threshold using the weight as the level.
    pub fn threshold(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        Threshold::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply gaussian-style blur. Weight is pass count.
    pub fn gauss(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        GaussBlur::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply motion blur. Weight is distance.
    pub fn motion(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        MotionBlur::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply noise. Weight is amount.
    pub fn noise(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        AddNoise::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply emboss. Weight is ignored (effect has no strength).
    pub fn emboss(
        frame: &mut VideoFrame,
        _weights: &LiveWeights,
        _name: &str,
    ) -> Result<(), WeightedError> {
        Emboss::apply(frame).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply bevel. Weight is depth.
    pub fn bevel(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        Bevel::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply stroke. Weight is outline color.
    pub fn stroke(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        Stroke::apply(frame, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply gradient. Weight is the end value; start is 0.
    pub fn gradient(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        Gradient::apply(frame, 0, Self::strength(weights, name)?).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply checker pattern. Weight is the light value.
    pub fn pattern(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        Pattern::apply(frame, Self::strength(weights, name)?, 0, 2).map_err(|_| WeightedError::BadWeight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn missing() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        let w = LiveWeights::new();
        assert!(WeightedEffect::glow(&mut f, &w, "glow").is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(32);
        f.planes[0][1 * 4 + 1] = 200;
        let mut w = LiveWeights::new();
        w.set("glow", 50.0).unwrap();
        WeightedEffect::glow(&mut f, &w, "glow").unwrap();
        assert!(f.planes[0][1 * 4 + 1] > 200);
    }

    #[test]
    fn shadow_and_sharpen() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0][0] = 255;
        let mut w = LiveWeights::new();
        w.set("shadow", 100.0).unwrap();
        WeightedEffect::shadow(&mut f, &w, "shadow").unwrap();
        assert!(f.planes[0][1 * 4 + 1] < 255);

        let mut f2 = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f2.planes[0].fill(32);
        f2.planes[0][1 * 4 + 1] = 200;
        w.set("sharp", 128.0).unwrap();
        WeightedEffect::sharpen(&mut f2, &w, "sharp").unwrap();
        assert!(f2.planes[0][1 * 4 + 1] >= 200);
    }

    #[test]
    fn suite() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        f.planes[0].fill(64);
        let mut w = LiveWeights::new();
        w.set("hue", 10.0).unwrap();
        WeightedEffect::hue_sat(&mut f, &w, "hue").unwrap();
        assert_eq!(f.planes[0][0], 74);
        w.set("exp", 1.0).unwrap();
        WeightedEffect::exposure(&mut f, &w, "exp").unwrap();
        assert_eq!(f.planes[0][0], 148);
        w.set("thr", 100.0).unwrap();
        WeightedEffect::threshold(&mut f, &w, "thr").unwrap();
        assert_eq!(f.planes[0][0], 255);
    }
}
