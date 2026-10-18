//! OBS x264 defaults (shippable metrics).
//!
//! Parsed from plugins/obs-x264/obs-x264.c obs_x264_defaults.
//! bitrate = 6000 kbps, rate_control = CBR, preset = veryfast,
//! crf = 23, keyint_sec = 0, buffer_size = 6000.
//! LiveWeights override any of these when set.
//! No unwraps.
//!
//! Differential: a higher bitrate setting produces a higher value.

/// Default bitrate in kbps (OBS "bitrate").
pub const OBS_BITRATE_KBPS: u32 = 6000;

/// Default buffer size in kbps (OBS "buffer_size").
pub const OBS_BUFFER_SIZE_KBPS: u32 = 6000;

/// Default CRF when rate control is CRF (OBS "crf").
pub const OBS_CRF: u32 = 23;

/// Default keyint in seconds. 0 means derive from fps (OBS "keyint_sec").
pub const OBS_KEYINT_SEC: u32 = 0;

/// Default rate control (OBS "rate_control").
pub const OBS_RATE_CONTROL: &str = "CBR";

/// Default preset (OBS "preset").
pub const OBS_PRESET: &str = "veryfast";

/// Default profile. Empty means auto (OBS "profile").
pub const OBS_PROFILE: &str = "";

#[derive(Clone, Debug)]
pub struct ObsEncoderDefaults {
    pub bitrate_kbps: u32,
    pub buffer_size_kbps: u32,
    pub crf: u32,
    pub keyint_sec: u32,
    pub rate_control: String,
    pub preset: String,
}

impl Default for ObsEncoderDefaults {
    fn default() -> Self {
        Self {
            bitrate_kbps: OBS_BITRATE_KBPS,
            buffer_size_kbps: OBS_BUFFER_SIZE_KBPS,
            crf: OBS_CRF,
            keyint_sec: OBS_KEYINT_SEC,
            rate_control: OBS_RATE_CONTROL.to_string(),
            preset: OBS_PRESET.to_string(),
        }
    }
}

impl ObsEncoderDefaults {
    /// Build from LiveWeights, falling back to OBS defaults.
    pub fn from_weights(weights: &crate::live_weights::LiveWeights) -> Self {
        let mut d = Self::default();
        if let Ok(v) = weights.get("h264.bitrate") {
            if v > 0.0 {
                d.bitrate_kbps = v as u32;
            }
        }
        if let Ok(v) = weights.get("h264.crf") {
            d.crf = (v as u32).clamp(0, 51);
        }
        if let Ok(v) = weights.get("h264.keyint_sec") {
            d.keyint_sec = v as u32;
        }
        if let Ok(v) = weights.get("h264.buffer_size") {
            if v > 0.0 {
                d.buffer_size_kbps = v as u32;
            }
        }
        d
    }

    /// GOP size derived the OBS way: keyint_sec * fps, or 2 * fps if keyint_sec is 0.
    pub fn gop(&self, fps: f32) -> u32 {
        let fps = if fps > 0.0 { fps } else { 30.0 };
        if self.keyint_sec == 0 {
            (fps * 2.0) as u32
        } else {
            (self.keyint_sec as f32 * fps) as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_weights::LiveWeights;

    #[test]
    fn defaults_match_obs() {
        let d = ObsEncoderDefaults::default();
        assert_eq!(d.bitrate_kbps, 6000);
        assert_eq!(d.crf, 23);
        assert_eq!(d.rate_control, "CBR");
        assert_eq!(d.preset, "veryfast");
    }

    #[test]
    fn weights_override() {
        let mut w = LiveWeights::new();
        w.set("h264.bitrate", 2500.0).unwrap();
        w.set("h264.crf", 18.0).unwrap();
        let d = ObsEncoderDefaults::from_weights(&w);
        assert_eq!(d.bitrate_kbps, 2500);
        assert_eq!(d.crf, 18);
    }

    #[test]
    fn gop_from_fps() {
        let d = ObsEncoderDefaults::default();
        assert_eq!(d.gop(30.0), 60);
        assert_eq!(d.gop(60.0), 120);
    }

    #[test]
    fn differential() {
        let a = ObsEncoderDefaults::default();
        let mut w = LiveWeights::new();
        w.set("h264.bitrate", 8000.0).unwrap();
        let b = ObsEncoderDefaults::from_weights(&w);
        assert!(b.bitrate_kbps > a.bitrate_kbps);
    }
}
