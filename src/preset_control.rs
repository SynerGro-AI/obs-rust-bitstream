//! Safe pure PresetControl (speed/quality preset core).
//!
//! Holds a preset name and writes it to LiveWeights
//! under "h264.preset". No unwraps.
//!
//! Differential: set updates the weight.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetKind {
    Fast,
    Balanced,
    Quality,
}

impl PresetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PresetKind::Fast => "fast",
            PresetKind::Balanced => "balanced",
            PresetKind::Quality => "quality",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fast" => Some(PresetKind::Fast),
            "balanced" => Some(PresetKind::Balanced),
            "quality" => Some(PresetKind::Quality),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PresetControl {
    kind: PresetKind,
}

impl PresetControl {
    pub fn new() -> Self {
        Self {
            kind: PresetKind::Balanced,
        }
    }

    pub fn kind(&self) -> PresetKind {
        self.kind
    }

    /// Set the preset and write the weight.
    pub fn set(&mut self, kind: PresetKind, weights: &mut LiveWeights) {
        self.kind = kind;
        let _ = weights.set("h264.preset", match kind {
            PresetKind::Fast => 0.0,
            PresetKind::Balanced => 1.0,
            PresetKind::Quality => 2.0,
        });
    }

    /// Read the weight into the control.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(v) = weights.get("h264.preset") {
            self.kind = match v as u8 {
                0 => PresetKind::Fast,
                2 => PresetKind::Quality,
                _ => PresetKind::Balanced,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut p = PresetControl::new();
        let mut w = LiveWeights::new();
        p.set(PresetKind::Quality, &mut w);
        assert_eq!(p.kind(), PresetKind::Quality);
        assert_eq!(w.get("h264.preset").unwrap(), 2.0);
        p.from_weights(&w);
        assert_eq!(p.kind(), PresetKind::Quality);
    }
}
