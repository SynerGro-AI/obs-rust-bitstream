//! Safe pure EncoderControls (bitrate + quality panel).
//!
//! Holds a BitrateSlider and a QualitySlider.
//! Apply writes both weights. No unwraps.
//!
//! Differential: apply sets both weights.

use crate::bitrate_slider::{BitrateSlider, SliderError};
use crate::live_weights::LiveWeights;
use crate::quality_slider::{QualityError, QualitySlider};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlError {
    Bitrate(SliderError),
    Quality(QualityError),
}

pub struct EncoderControls {
    bitrate: BitrateSlider,
    quality: QualitySlider,
}

impl EncoderControls {
    pub fn new() -> Result<Self, ControlError> {
        let bitrate = BitrateSlider::new(0.0, 20_000_000.0).map_err(ControlError::Bitrate)?;
        Ok(Self {
            bitrate,
            quality: QualitySlider::new(),
        })
    }

    pub fn bitrate(&self) -> f32 {
        self.bitrate.value()
    }

    pub fn quality(&self) -> f32 {
        self.quality.value()
    }

    /// Set bitrate and write the weight.
    pub fn set_bitrate(&mut self, value: f32, weights: &mut LiveWeights) -> Result<(), ControlError> {
        self.bitrate.set(value, weights).map_err(ControlError::Bitrate)
    }

    /// Set QP and write the weight.
    pub fn set_quality(&mut self, value: f32, weights: &mut LiveWeights) -> Result<(), ControlError> {
        self.quality.set(value, weights).map_err(ControlError::Quality)
    }

    /// Write both current values.
    pub fn apply(&self, weights: &mut LiveWeights) -> Result<(), ControlError> {
        weights
            .set("h264.bitrate", self.bitrate.value())
            .map_err(|_| ControlError::Bitrate(SliderError::Negative))?;
        weights
            .set("h264.qp", self.quality.value())
            .map_err(|_| ControlError::Quality(QualityError::OutOfRange))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut c = EncoderControls::new().unwrap();
        let mut w = LiveWeights::new();
        c.set_bitrate(2_000_000.0, &mut w).unwrap();
        c.set_quality(30.0, &mut w).unwrap();
        c.apply(&mut w).unwrap();
        assert_eq!(w.get("h264.bitrate").unwrap(), 2_000_000.0);
        assert_eq!(w.get("h264.qp").unwrap(), 30.0);
    }
}
