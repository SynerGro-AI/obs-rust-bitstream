//! Safe pure SystemStatus (GPU + encoder + bundle core).
//!
//! Combines GPU info, encoder choice, and bundle
//! stats into one report. No unwraps.
//!
//! Differential: no GPU reports software.

use crate::av_bundle::AvBundle;
use crate::encoder_select::{EncoderChoice, EncoderSelect};
use crate::gpu_detect::GpuDetect;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct SystemStatus {
    pub gpu_present: bool,
    pub encoder: EncoderChoice,
    pub video_bytes: usize,
    pub audio_samples: usize,
    pub fps: f32,
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
        }
    }

    /// Short summary.
    pub fn summary(&self) -> String {
        format!(
            "gpu={} encoder={} video={} audio={} fps={}",
            self.gpu_present,
            self.encoder.as_str(),
            self.video_bytes,
            self.audio_samples,
            self.fps
        )
    }

    /// Write the summary to a writer.
    pub fn write<W: Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        writeln!(out, "{}", self.summary())
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
        let mut buf = Vec::new();
        s.write(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("software"));
    }
}
