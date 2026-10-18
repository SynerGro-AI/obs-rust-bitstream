//! Safe pure TickLoop (N capture ticks).
//!
//! Runs CaptureTick N times and returns the final
//! TickStatus. Zero ticks return a stopped report.
//! No unwraps.
//!
//! Differential: more ticks produce more bytes.

use crate::audio_source_list::AudioSourceList;
use crate::capture::Capture;
use crate::capture_tick::CaptureTick;
use crate::h264_codec::H264Codec;
use crate::output_info::OutputInfo;
use crate::tick_status::TickStatus;

pub struct TickLoop;

impl TickLoop {
    /// Run `count` ticks. Returns the final status.
    /// A stopped output or zero count returns a zero report.
    pub fn run(
        capture: &mut dyn Capture,
        codec: &mut H264Codec,
        audio: &mut AudioSourceList,
        output: &mut OutputInfo,
        count: usize,
    ) -> TickStatus {
        if count == 0 || !output.is_running() {
            return TickStatus {
                packets: 0,
                bytes: output.bytes(),
                running: output.is_running(),
            };
        }
        let mut last = TickStatus {
            packets: 0,
            bytes: 0,
            running: true,
        };
        for i in 0..count {
            last = TickStatus::from(capture, codec, audio, output, i as i64);
        }
        last
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
    fn loop_counts() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let s = TickLoop::run(&mut cap, &mut codec, &mut audio, &mut out, 3);
        let _ = codec.flush();
        assert!(s.packets >= 1);
        assert!(s.bytes > 0);
        assert!(s.running);
    }

    #[test]
    fn zero_is_zero() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let s = TickLoop::run(&mut cap, &mut codec, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert_eq!(s.packets, 0);
    }

    #[test]
    fn differential() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let a = TickLoop::run(&mut cap, &mut codec, &mut audio, &mut out, 1);
        let mut out2 = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        out2.start();
        let b = TickLoop::run(&mut cap, &mut codec, &mut audio, &mut out2, 3);
        let _ = codec.flush();
        assert!(b.bytes > a.bytes);
    }
}
