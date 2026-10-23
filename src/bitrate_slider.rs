//! Safe pure BitrateSlider (UI control core).
//!
//! Holds a bitrate value and writes it to LiveWeights
//! under "h264.bitrate". No unwraps.
//!
//! Differential: set updates the weight.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SliderError {
    Negative,
}

pub struct BitrateSlider {
    value: f32,
    min: f32,
    max: f32,
}

impl BitrateSlider {
    pub fn new(min: f32, max: f32) -> Result<Self, SliderError> {
        if min < 0.0 || max < min {
            return Err(SliderError::Negative);
        }
        Ok(Self {
            value: min,
            min,
            max,
        })
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set the slider and write the weight.
    pub fn set(&mut self, value: f32, weights: &mut LiveWeights) -> Result<(), SliderError> {
        if value < self.min || value > self.max {
            return Err(SliderError::Negative);
        }
        self.value = value;
        weights
            .set("h264.bitrate", value)
            .map_err(|_| SliderError::Negative)?;
        Ok(())
    }

    /// Read the weight into the slider.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(v) = weights.get("h264.bitrate") {
            if v >= self.min && v <= self.max {
                self.value = v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut s = BitrateSlider::new(0.0, 10_000_000.0).unwrap();
        let mut w = LiveWeights::new();
        s.set(4_000_000.0, &mut w).unwrap();
        assert_eq!(s.value(), 4_000_000.0);
        assert_eq!(w.get("h264.bitrate").unwrap(), 4_000_000.0);
        assert!(s.set(-1.0, &mut w).is_err());
    }
}
