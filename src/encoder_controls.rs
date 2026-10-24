//! Safe pure EncoderControls (bitrate + quality panel).
//!
//! Holds a BitrateSlider and a QualitySlider.
//! Apply writes both weights. No unwraps.
//!
//! Differential: apply sets both weights.

use crate::bitrate_slider::{BitrateSlider, SliderError};
use crate::cancel_button::CancelButton;
use crate::encode_cancel::EncodeCancel;
use crate::gop_slider::{GopError, GopSlider};
use crate::live_weights::LiveWeights;
use crate::preset_control::PresetControl;
use crate::quality_slider::{QualityError, QualitySlider};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlError {
    Bitrate(SliderError),
    Quality(QualityError),
    Gop(GopError),
}

pub struct EncoderControls {
    bitrate: BitrateSlider,
    quality: QualitySlider,
    gop: GopSlider,
    preset: PresetControl,
    cancel: CancelButton,
}

impl EncoderControls {
    pub fn new() -> Result<Self, ControlError> {
        let bitrate = BitrateSlider::new(0.0, 20_000_000.0).map_err(ControlError::Bitrate)?;
        Ok(Self {
            bitrate,
            quality: QualitySlider::new(),
            gop: GopSlider::new(),
            preset: PresetControl::new(),
            cancel: CancelButton::new(),
        })
    }

    pub fn bitrate(&self) -> f32 {
        self.bitrate.value()
    }

    pub fn quality(&self) -> f32 {
        self.quality.value()
    }

    /// Set bitrate and write the weight.
    pub fn set_bitrate(
        &mut self,
        value: f32,
        weights: &mut LiveWeights,
    ) -> Result<(), ControlError> {
        self.bitrate
            .set(value, weights)
            .map_err(ControlError::Bitrate)
    }

    pub fn gop(&self) -> f32 {
        self.gop.value()
    }

    /// Set GOP size and write the weight.
    pub fn set_quality(
        &mut self,
        value: f32,
        weights: &mut LiveWeights,
    ) -> Result<(), ControlError> {
        self.quality
            .set(value, weights)
            .map_err(ControlError::Quality)
    }

    pub fn preset(&self) -> crate::preset_control::PresetKind {
        self.preset.kind()
    }

    pub fn cancel_pressed(&self) -> bool {
        self.cancel.is_pressed()
    }

    /// Press cancel and set the flag.
    pub fn press_cancel(&mut self, cancel: &mut EncodeCancel) {
        self.cancel.press(cancel);
    }

    /// Release cancel and clear the flag.
    pub fn release_cancel(&mut self, cancel: &mut EncodeCancel) {
        self.cancel.release(cancel);
    }

    /// Current settings as a short string.
    pub fn summary(&self) -> String {
        format!(
            "bitrate={} qp={} gop={} preset={:?} cancel={}",
            self.bitrate.value(),
            self.quality.value(),
            self.gop.value(),
            self.preset.kind(),
            self.cancel.is_pressed()
        )
    }

    /// Restore defaults and write them.
    pub fn reset(&mut self, weights: &mut LiveWeights) -> Result<(), ControlError> {
        self.set_bitrate(0.0, weights)?;
        self.set_quality(28.0, weights)?;
        self.set_gop(30.0, weights)?;
        self.set_preset(crate::preset_control::PresetKind::Balanced, weights);
        Ok(())
    }

    /// Set preset and write the weight.
    pub fn set_preset(
        &mut self,
        kind: crate::preset_control::PresetKind,
        weights: &mut LiveWeights,
    ) {
        self.preset.set(kind, weights);
    }

    /// Write both current values.
    pub fn set_gop(&mut self, value: f32, weights: &mut LiveWeights) -> Result<(), ControlError> {
        self.gop.set(value, weights).map_err(ControlError::Gop)
    }

    /// Write both current values.
    pub fn apply(&self, weights: &mut LiveWeights) -> Result<(), ControlError> {
        weights
            .set("h264.bitrate", self.bitrate.value())
            .map_err(|_| ControlError::Bitrate(SliderError::Negative))?;
        weights
            .set("h264.qp", self.quality.value())
            .map_err(|_| ControlError::Quality(QualityError::OutOfRange))?;
        weights
            .set("h264.gop", self.gop.value())
            .map_err(|_| ControlError::Gop(GopError::Zero))?;
        let preset_val = match self.preset.kind() {
            crate::preset_control::PresetKind::Fast => 0.0,
            crate::preset_control::PresetKind::Balanced => 1.0,
            crate::preset_control::PresetKind::Quality => 2.0,
        };
        let _ = weights.set("h264.preset", preset_val);
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
        assert!(c.summary().contains("qp=30"));
    }
}
