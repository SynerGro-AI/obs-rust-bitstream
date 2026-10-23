//! Safe pure QualitySlider (QP control core).
//!
//! Holds a QP value (0..=51) and writes it to
//! LiveWeights under "h264.qp". No unwraps.
//!
//! Differential: set updates the weight.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QualityError {
    OutOfRange,
}

pub struct QualitySlider {
    value: f32,
}

impl QualitySlider {
    pub fn new() -> Self {
        Self { value: 28.0 }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set QP and write the weight. Valid range 0..=51.
    pub fn set(&mut self, value: f32, weights: &mut LiveWeights) -> Result<(), QualityError> {
        if !(0.0..=51.0).contains(&value) {
            return Err(QualityError::OutOfRange);
        }
        self.value = value;
        weights
            .set("h264.qp", value)
            .map_err(|_| QualityError::OutOfRange)?;
        Ok(())
    }

    /// Read the weight into the slider.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(v) = weights.get("h264.qp") {
            if (0.0..=51.0).contains(&v) {
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
        let mut s = QualitySlider::new();
        let mut w = LiveWeights::new();
        s.set(32.0, &mut w).unwrap();
        assert_eq!(s.value(), 32.0);
        assert_eq!(w.get("h264.qp").unwrap(), 32.0);
        assert!(s.set(60.0, &mut w).is_err());
    }
}
