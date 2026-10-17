//! Safe pure SystemStatus (GPU + encoder + bundle core).
//!
//! Combines GPU info, encoder choice, and bundle
//! stats into one report. No unwraps.
//!
//! Differential: no GPU reports software.

use crate::av_bundle::AvBundle;
use crate::encoder_select::{EncoderChoice, EncoderSelect};
use crate::gpu_detect::GpuDetect;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct SystemStatus {
    pub gpu_present: bool,
    pub encoder: EncoderChoice,
    pub video_bytes: usize,
    pub audio_samples: usize,
    pub fps: f32,
    pub encode_ms: f32,
    pub stamped: bool,
    pub version: String,
    pub built: String,
    pub stamp_opacity: f32,
    pub captured_frames: usize,
}

impl SystemStatus {
    /// Build a status from a bundle.
    pub fn from_bundle(bundle: &AvBundle) -> Self {
        let info = GpuDetect::detect();
        Self {
            gpu_present: info.is_present(),
            encoder: EncoderSelect::choose(),
            video_bytes: bundle.video_len(),
            audio_samples: bundle.audio_len(),
            fps: bundle.fps(),
            encode_ms: 0.0,
            stamped: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
            built: option_env!("BUILD_TIME").unwrap_or("unknown").to_string(),
            stamp_opacity: 1.0,
            captured_frames: 0,
        }
    }

    pub fn set_encode_ms(&mut self, ms: f32) {
        if ms >= 0.0 {
            self.encode_ms = ms;
        }
    }

    pub fn set_stamped(&mut self, stamped: bool) {
        self.stamped = stamped;
    }

    pub fn set_version(&mut self, v: &str) {
        self.version = v.to_string();
    }

    pub fn set_stamp_opacity(&mut self, o: f32) {
        self.stamp_opacity = o.clamp(0.0, 1.0);
    }

    pub fn set_captured_frames(&mut self, n: usize) {
        self.captured_frames = n;
    }

    /// Short summary.
    pub fn summary(&self) -> String {
        format!(
            "gpu={} encoder={} video={} audio={} fps={} encode_ms={:.1} stamped={} version={} built={} opacity={:.1} captured={}",
            self.gpu_present,
            self.encoder.as_str(),
            self.video_bytes,
            self.audio_samples,
            self.fps,
            self.encode_ms,
            self.stamped,
            self.version,
            self.built,
            self.stamp_opacity,
            self.captured_frames
        )
    }

    /// Write the summary to a writer.
    pub fn write<W: Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        writeln!(out, "{}", self.summary())
    }

    /// Read a summary back. Returns the encoder string.
    pub fn read_encoder<R: Read>(mut input: R) -> Result<String, std::io::Error> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        for part in buf.split_whitespace() {
            if let Some(val) = part.strip_prefix("encoder=") {
                return Ok(val.to_string());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "no encoder",
        ))
    }

    /// True if the status encoder matches the bundle encoder.
    pub fn matches_bundle(&self, bundle: &AvBundle) -> bool {
        self.encoder == bundle.encoder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let b = AvBundle::new();
        let s = SystemStatus::from_bundle(&b);
        assert!(!s.gpu_present);
        assert_eq!(s.encoder, EncoderChoice::Software);
        assert!(s.summary().contains("software"));
        let mut s2 = s.clone();
        s2.set_encode_ms(7.5);
        assert!(s2.summary().contains("encode_ms=7.5"));
        s2.set_stamped(true);
        assert!(s2.summary().contains("stamped=true"));
        s2.set_version("0.2.0");
        assert!(s2.summary().contains("version=0.2.0"));
        assert!(s2.summary().contains("built="));
        s2.set_stamp_opacity(0.5);
        assert!(s2.summary().contains("opacity=0.5"));
        s2.set_captured_frames(4);
        assert!(s2.summary().contains("captured=4"));
        let mut buf = Vec::new();
        s.write(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("software"));
        let enc = SystemStatus::read_encoder(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(enc, "software");
        assert!(s.matches_bundle(&b));
    }
}
