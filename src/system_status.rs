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
    pub source: String,
    pub timestamp: u64,
    pub sources_available: usize,
    pub source_summary: String,
    pub capture_fps: f32,
    pub capture_faster: bool,
    pub bottleneck: String,
    pub recommendation: String,
    pub healthy: bool,
    pub target_fps: f32,
    pub hit_target: bool,
    pub achieved_fps: f32,
    pub pacing: String,
    pub source_count: usize,
    pub active_source: String,
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
            source: "none".to_string(),
            timestamp: 0,
            sources_available: 0,
            source_summary: String::new(),
            capture_fps: 0.0,
            capture_faster: false,
            bottleneck: String::new(),
            recommendation: String::new(),
            healthy: false,
            target_fps: 0.0,
            hit_target: false,
            achieved_fps: 0.0,
            pacing: String::new(),
            source_count: 0,
            active_source: String::new(),
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

    pub fn set_source(&mut self, name: &str) {
        self.source = name.to_string();
    }

    pub fn set_timestamp(&mut self, ts: u64) {
        self.timestamp = ts;
    }

    pub fn set_sources_available(&mut self, n: usize) {
        self.sources_available = n;
    }

    pub fn set_source_summary(&mut self, s: &str) {
        self.source_summary = s.to_string();
    }

    pub fn set_capture_fps(&mut self, fps: f32) {
        if fps >= 0.0 {
            self.capture_fps = fps;
        }
    }

    pub fn set_capture_faster(&mut self, faster: bool) {
        self.capture_faster = faster;
    }

    pub fn set_bottleneck(&mut self, side: &str) {
        self.bottleneck = side.to_string();
    }

    pub fn set_recommendation(&mut self, rec: &str) {
        self.recommendation = rec.to_string();
    }

    pub fn set_healthy(&mut self, healthy: bool) {
        self.healthy = healthy;
    }

    pub fn set_target_fps(&mut self, fps: f32) {
        if fps >= 0.0 {
            self.target_fps = fps;
        }
    }

    pub fn set_hit_target(&mut self, hit: bool) {
        self.hit_target = hit;
    }

    pub fn set_achieved_fps(&mut self, fps: f32) {
        if fps >= 0.0 {
            self.achieved_fps = fps;
        }
    }

    pub fn set_pacing(&mut self, s: &str) {
        self.pacing = s.to_string();
    }

    pub fn set_source_count(&mut self, n: usize) {
        self.source_count = n;
    }

    pub fn set_active_source(&mut self, name: &str) {
        self.active_source = name.to_string();
    }

    /// Current Unix time.
    pub fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Short summary.
    pub fn summary(&self) -> String {
        format!(
            "gpu={} encoder={} video={} audio={} fps={} encode_ms={:.1} stamped={} version={} built={} opacity={:.1} captured={} source={} time={} available={} summary={} capture_fps={:.0} capture_faster={} bottleneck={} recommend={} healthy={} target={:.0} hit={} achieved={:.0} pacing={} sources={} active={}",
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
            self.captured_frames,
            self.source,
            self.timestamp,
            self.sources_available,
            self.source_summary,
            self.capture_fps,
            self.capture_faster,
            self.bottleneck,
            self.recommendation,
            self.healthy,
            self.target_fps,
            self.hit_target,
            self.achieved_fps,
            self.pacing,
            self.source_count,
            self.active_source
        )
    }

    /// One readable line.
    pub fn one_line(&self) -> String {
        format!(
            "{} | {} fps | {} | {}",
            self.source_summary,
            self.capture_fps as u32,
            self.bottleneck,
            if self.healthy { "healthy" } else { "slow" }
        )
    }

    /// Write the summary to a writer.
    pub fn write<W: Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        writeln!(out, "{}", self.summary())?;
        writeln!(out, "{}", self.one_line())
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

    /// Read the one-line form. Returns the second line.
    pub fn read_one_line<R: Read>(mut input: R) -> Result<String, std::io::Error> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        let lines: Vec<&str> = buf.lines().collect();
        if lines.len() >= 2 {
            Ok(lines[1].to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "no one-line",
            ))
        }
    }

    /// True if the read-back matches this one-line.
    pub fn matches_one_line(&self, read_back: &str) -> bool {
        self.one_line() == read_back
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
        s2.set_source("synthetic");
        assert!(s2.summary().contains("source=synthetic"));
        s2.set_sources_available(3);
        assert!(s2.summary().contains("available=3"));
        s2.set_source_summary("synthetic 16x16");
        assert!(s2.summary().contains("summary=synthetic"));
        s2.set_capture_fps(30.0);
        assert!(s2.summary().contains("capture_fps=30"));
        s2.set_capture_faster(true);
        assert!(s2.summary().contains("capture_faster=true"));
        s2.set_bottleneck("encode");
        assert!(s2.summary().contains("bottleneck=encode"));
        s2.set_recommendation("lower quality");
        assert!(s2.summary().contains("recommend=lower"));
        s2.set_healthy(true);
        assert!(s2.summary().contains("healthy=true"));
        s2.set_target_fps(30.0);
        assert!(s2.summary().contains("target=30"));
        s2.set_hit_target(true);
        assert!(s2.summary().contains("hit=true"));
        s2.set_achieved_fps(29.5);
        assert!(s2.summary().contains("achieved=30"));
        s2.set_pacing("target 30 | hit");
        assert!(s2.summary().contains("pacing=target"));
        s2.set_source_count(2);
        assert!(s2.summary().contains("sources=2"));
        s2.set_active_source("synthetic");
        assert!(s2.summary().contains("active=synthetic"));
        assert!(s2.one_line().contains("healthy"));
        let mut buf2 = Vec::new();
        s2.write(&mut buf2).unwrap();
        let line = SystemStatus::read_one_line(std::io::Cursor::new(&buf2)).unwrap();
        assert!(line.contains("healthy"));
        assert!(s2.matches_one_line(&line));
        s2.set_timestamp(SystemStatus::now());
        assert!(s2.summary().contains("time="));
        let mut buf = Vec::new();
        s.write(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("software"));
        let enc = SystemStatus::read_encoder(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(enc, "software");
        assert!(s.matches_bundle(&b));
    }
}
