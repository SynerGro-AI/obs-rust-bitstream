//! Safe pure WeightedEffect (effect that reads live weights).
//!
//! Applies an effect using the strength stored in LiveWeights.
//! Missing weight is an error. No unwraps.
//!
//! Differential: a weight of 50 lifts an edge pixel.

use crate::drop_shadow::DropShadow;
use crate::inner_glow::InnerGlow;
use crate::live_weights::{LiveWeights, WeightError};
use crate::sharpen::Sharpen;
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
    /// Apply inner glow using the weight named `name`.
    /// The weight is treated as 0..=255 strength.
    pub fn glow(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        let v = weights.get(name)?;
        if !(0.0..=255.0).contains(&v) {
            return Err(WeightedError::BadWeight);
        }
        let strength = v as u8;
        InnerGlow::apply(frame, strength).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply drop shadow using the weight named `name`.
    pub fn shadow(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        let v = weights.get(name)?;
        if !(0.0..=255.0).contains(&v) {
            return Err(WeightedError::BadWeight);
        }
        DropShadow::apply(frame, 2, 2, v as u8).map_err(|_| WeightedError::BadWeight)
    }

    /// Apply sharpen using the weight named `name`.
    pub fn sharpen(
        frame: &mut VideoFrame,
        weights: &LiveWeights,
        name: &str,
    ) -> Result<(), WeightedError> {
        let v = weights.get(name)?;
        if !(0.0..=255.0).contains(&v) {
            return Err(WeightedError::BadWeight);
        }
        Sharpen::apply(frame, v as u8).map_err(|_| WeightedError::BadWeight)
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
}
