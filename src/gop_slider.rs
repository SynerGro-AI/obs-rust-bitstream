//! Safe pure GopSlider (keyframe interval core).
//!
//! Holds a GOP size and writes it to LiveWeights
//! under "h264.gop". No unwraps.
//!
//! Differential: set updates the weight.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GopError {
    Zero,
}

pub struct GopSlider {
    value: f32,
}

impl GopSlider {
    pub fn new() -> Self {
        Self { value: 30.0 }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set GOP size and write the weight. Must be > 0.
    pub fn set(&mut self, value: f32, weights: &mut LiveWeights) -> Result<(), GopError> {
        if value < 1.0 {
            return Err(GopError::Zero);
        }
        self.value = value;
        weights.set("h264.gop", value).map_err(|_| GopError::Zero)?;
        Ok(())
    }

    /// Read the weight into the slider.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(v) = weights.get("h264.gop") {
            if v >= 1.0 {
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
        let mut s = GopSlider::new();
        let mut w = LiveWeights::new();
        s.set(60.0, &mut w).unwrap();
        assert_eq!(s.value(), 60.0);
        assert_eq!(w.get("h264.gop").unwrap(), 60.0);
        assert!(s.set(0.0, &mut w).is_err());
    }
}
