//! Safe pure TickStatus (report one capture tick).
//!
//! Runs one CaptureTick and returns a one-line report
//! of packets accepted and total bytes. No unwraps.
//!
//! Differential: a running tick reports more bytes than a stopped one.

use crate::audio_source_list::AudioSourceList;
use crate::capture::Capture;
use crate::capture_tick::CaptureTick;
use crate::h264_codec::H264Codec;
use crate::output_info::OutputInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickStatus {
    pub packets: usize,
    pub bytes: u64,
    pub running: bool,
}

impl TickStatus {
    /// Run one tick and report the result.
    pub fn from(
        capture: &mut dyn Capture,
        codec: &mut H264Codec,
        audio: &mut AudioSourceList,
        output: &mut OutputInfo,
        pts: i64,
    ) -> Self {
        let packets = CaptureTick::tick(capture, codec, audio, output, pts);
        Self {
            packets,
            bytes: output.bytes(),
            running: output.is_running(),
        }
    }

    /// One line: "packets=N bytes=N running=true/false"
    pub fn one_line(&self) -> String {
        format!(
            "packets={} bytes={} running={}",
            self.packets, self.bytes, self.running
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;
    use crate::capture::SyntheticCapture;
    use crate::live_weights::LiveWeights;
    use crate::output_info::{OutputInfo, OUTPUT_AV};

    fn setup() -> (SyntheticCapture, H264Codec, AudioSourceList, OutputInfo) {
        let cap = SyntheticCapture::new(16, 16);
        let weights = LiveWeights::new();
        let codec = H264Codec::new(16, 16, &weights).unwrap();
        let mut audio = AudioSourceList::new();
        audio.add("tone", AudioSource::synthetic(48000, 1));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        out.start();
        (cap, codec, audio, out)
    }

    #[test]
    fn reports_bytes() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let s = TickStatus::from(&mut cap, &mut codec, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert!(s.packets >= 1);
        assert!(s.bytes > 0);
        assert!(s.running);
        assert!(s.one_line().contains("packets="));
    }

    #[test]
    fn stopped_is_zero() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        out.stop();
        let s = TickStatus::from(&mut cap, &mut codec, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert_eq!(s.packets, 0);
        assert!(!s.running);
    }

    #[test]
    fn differential() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let a = TickStatus::from(&mut cap, &mut codec, &mut audio, &mut out, 0);
        let mut out2 = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let b = TickStatus::from(&mut cap, &mut codec, &mut audio, &mut out2, 0);
        let _ = codec.flush();
        assert!(a.bytes > b.bytes);
    }
}
