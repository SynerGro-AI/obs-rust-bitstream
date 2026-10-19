//! Safe pure UiStatus (status dashboard).
//!
//! Aggregates capture, audio, stream, and RTMP open
//! into one view. No unwraps.
//!
//! Differential: a live capture reports live=true.

use crate::audio_backend_status::AudioBackendStatus;
use crate::rtmp_open_status::RtmpOpenStatus;
use crate::stream_status::StreamStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiStatus {
    pub capture: String,
    pub capture_live: bool,
    pub audio_backend: String,
    pub audio_live: bool,
    pub streaming: bool,
    pub rtmp_bytes: u64,
}

impl UiStatus {
    /// Build from the existing status pieces.
    pub fn from(
        capture_kind: &str,
        capture_live: bool,
        audio: &AudioBackendStatus,
        stream: &StreamStatus,
        open: &RtmpOpenStatus,
    ) -> Self {
        Self {
            capture: capture_kind.to_string(),
            capture_live,
            audio_backend: audio.backend.clone(),
            audio_live: audio.live,
            streaming: stream.streaming,
            rtmp_bytes: open.bytes,
        }
    }

    /// Multi-line dashboard.
    pub fn view(&self) -> String {
        format!(
            "capture={} live={}\naudio={} live={}\nstreaming={}\nrtmp_bytes={}",
            self.capture,
            self.capture_live,
            self.audio_backend,
            self.audio_live,
            self.streaming,
            self.rtmp_bytes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let audio = AudioBackendStatus {
            backend: "alsa".to_string(),
            live: false,
        };
        let stream = StreamStatus {
            streaming: true,
            service: "file".to_string(),
            bitrate_kbps: 6000,
        };
        let open = RtmpOpenStatus::from(3060);
        let ui = UiStatus::from("synthetic", false, &audio, &stream, &open);
        assert!(!ui.capture_live);
        assert!(ui.streaming);
        assert!(ui.view().contains("rtmp_bytes=3060"));
    }

    #[test]
    fn live_capture() {
        let audio = AudioBackendStatus {
            backend: "cpal".to_string(),
            live: true,
        };
        let stream = StreamStatus {
            streaming: true,
            service: "twitch".to_string(),
            bitrate_kbps: 6000,
        };
        let open = RtmpOpenStatus::from(1000);
        let ui = UiStatus::from("platform", true, &audio, &stream, &open);
        assert!(ui.capture_live);
        assert!(ui.audio_live);
    }

    #[test]
    fn differential() {
        let audio = AudioBackendStatus {
            backend: "alsa".to_string(),
            live: false,
        };
        let stream = StreamStatus {
            streaming: false,
            service: String::new(),
            bitrate_kbps: 0,
        };
        let a = UiStatus::from("synthetic", false, &audio, &stream, &RtmpOpenStatus::from(100));
        let b = UiStatus::from("platform", true, &audio, &stream, &RtmpOpenStatus::from(3060));
        assert!(b.rtmp_bytes > a.rtmp_bytes);
        assert!(!a.capture_live && b.capture_live);
    }
}
