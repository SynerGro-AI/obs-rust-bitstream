//! Safe pure AvTick (one video + one audio → output).
//!
//! Pulls one H.264 packet and one audio packet, interleaves
//! them by dts, and drains them into a running OutputInfo.
//! Missing packets are skipped. No unwraps.
//!
//! Differential: a tick with both packets counts more bytes
//! than a tick with only video.

use crate::audio_packet::AudioPacket;
use crate::audio_source_list::AudioSourceList;
use crate::encoder_packet::InterleaveQueue;
use crate::h264_codec::H264Codec;
use crate::output_feed::OutputFeed;
use crate::output_info::OutputInfo;
use crate::video_frame::VideoFrame;

pub struct AvTick;

impl AvTick {
    /// Encode one video frame and one audio pull, interleave, and feed.
    /// Returns the number of packets accepted by the output.
    /// A stopped output accepts zero.
    pub fn tick(
        codec: &mut H264Codec,
        frame: &VideoFrame,
        audio: &mut AudioSourceList,
        output: &mut OutputInfo,
        pts: i64,
    ) -> usize {
        if !output.is_running() {
            return 0;
        }
        let mut q = InterleaveQueue::new();
        if let Ok(Some(pkt)) = codec.encode_packet(frame, pts) {
            q.push(pkt);
        }
        if let Some(pkt) = AudioPacket::from_list(audio, 480, pts) {
            q.push(pkt);
        }
        OutputFeed::drain(&mut q, output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;
    use crate::live_weights::LiveWeights;
    use crate::output_info::{OutputInfo, OUTPUT_AV};
    use crate::video_fourcc::VideoFormat;
    use crate::video_frame::VideoFrame;

    fn setup() -> (H264Codec, VideoFrame, AudioSourceList, OutputInfo) {
        let weights = LiveWeights::new();
        let codec = H264Codec::new(16, 16, &weights).unwrap();
        let frame = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        let mut audio = AudioSourceList::new();
        audio.add("tone", AudioSource::synthetic(48000, 1));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        out.start();
        (codec, frame, audio, out)
    }

    #[test]
    fn tick_feeds() {
        let (mut codec, frame, mut audio, mut out) = setup();
        let n = AvTick::tick(&mut codec, &frame, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert!(n >= 1);
        assert!(out.bytes() > 0);
    }

    #[test]
    fn stopped_is_zero() {
        let (mut codec, frame, mut audio, mut out) = setup();
        out.stop();
        assert_eq!(AvTick::tick(&mut codec, &frame, &mut audio, &mut out, 0), 0);
        let _ = codec.flush();
    }

    #[test]
    fn differential() {
        let (mut codec, frame, mut audio, mut out) = setup();
        let before = out.bytes();
        AvTick::tick(&mut codec, &frame, &mut audio, &mut out, 0);
        let _ = codec.flush();
        assert!(out.bytes() > before);
    }
}
