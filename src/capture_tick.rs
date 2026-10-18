//! Safe pure CaptureTick (camera → interleaved output).
//!
//! Pulls one frame from a Capture and feeds it through
//! AvTick with the active audio source. No frame or a
//! stopped output returns zero. No unwraps.
//!
//! Differential: a running output counts bytes; a stopped one does not.

use crate::audio_source_list::AudioSourceList;
use crate::av_tick::AvTick;
use crate::capture::Capture;
use crate::h264_codec::H264Codec;
use crate::output_info::OutputInfo;

pub struct CaptureTick;

impl CaptureTick {
    /// Pull one frame from the capture and tick.
    /// Returns the number of packets accepted.
    /// Returns zero if the capture yields nothing or the output is stopped.
    pub fn tick(
        capture: &mut dyn Capture,
        codec: &mut H264Codec,
        audio: &mut AudioSourceList,
        output: &mut OutputInfo,
        pts: i64,
    ) -> usize {
        if !output.is_running() {
            return 0;
        }
        match capture.frame() {
            Some(frame) => AvTick::tick(codec, &frame, audio, output, pts),
            None => 0,
        }
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
    fn tick_feeds() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let n = CaptureTick::tick(&mut cap, &mut codec, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert!(n >= 1);
        assert!(out.bytes() > 0);
    }

    #[test]
    fn stopped_is_zero() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        out.stop();
        assert_eq!(
            CaptureTick::tick(&mut cap, &mut codec, &mut audio, &mut out, 0),
            0
        );
        let _ = codec.flush();
    }

    #[test]
    fn differential() {
        let (mut cap, mut codec, mut audio, mut out) = setup();
        let before = out.bytes();
        CaptureTick::tick(&mut cap, &mut codec, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert!(out.bytes() > before);
    }
}
