//! Safe pure StreamPayloadRound (write → read packets).
//!
//! Runs a StreamPayload and reads the packet count back.
//! Returns true if the count matches the header. No unwraps.
//!
//! Differential: more frames produce a higher or equal count.

use crate::capture::Capture;
use crate::h264_codec::H264Codec;
use crate::live_weights::LiveWeights;
use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::payload_dump::PayloadDump;
use crate::program_stream::ProgramStream;
use crate::runner::Runner;
use crate::stream_payload::{StreamPayload, StreamPayloadError};

pub struct StreamPayloadRound;

impl StreamPayloadRound {
    /// True if the written packet count matches the header.
    pub fn matches(
        runner: &mut Runner,
        stream: &mut ProgramStream,
        service: &mut OutputService,
        output: &OutputInfo,
        weights: &mut LiveWeights,
        codec: &mut H264Codec,
        capture: &mut dyn Capture,
        name: &str,
        id: &str,
        frames: usize,
    ) -> Result<bool, StreamPayloadError> {
        let (_, pk) = StreamPayload::run(
            runner, stream, service, output, weights, codec, capture, name, id, frames,
        )?;
        let header = PayloadDump::header(&pk);
        let packets = PayloadDump::packets(&pk);
        match (header, packets) {
            (Some(h), Some(p)) => Ok(h.count as usize == p.len()),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::{AudioSource, AudioSourceList};
    use crate::capture::SyntheticCapture;
    use crate::output_info::{OutputInfo, OUTPUT_AV};
    use crate::pipeline::Pipeline;

    fn runner() -> Runner {
        let mut weights = LiveWeights::new();
        let _ = weights.set("fps", 30.0);
        let _ = weights.set("h264.qp", 28.0);
        let pipe = Pipeline::new(16, 16, weights).expect("pipe");
        let cap = Box::new(SyntheticCapture::new(16, 16));
        let mut audio = AudioSourceList::new();
        let _ = audio.add("tone", AudioSource::synthetic(48000, 1));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).expect("out");
        out.start();
        Runner::new(pipe, cap, audio, out)
    }

    #[test]
    fn round_trips() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let mut codec = H264Codec::new(16, 16, &w).unwrap();
        let mut cap = SyntheticCapture::new(16, 16);
        assert!(StreamPayloadRound::matches(
            &mut r,
            &mut stream,
            &mut svc,
            &out,
            &mut w,
            &mut codec,
            &mut cap,
            "twitch",
            "rtmp",
            2,
        )
        .unwrap());
    }

    #[test]
    fn empty_rejected() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let mut codec = H264Codec::new(16, 16, &w).unwrap();
        let mut cap = SyntheticCapture::new(16, 16);
        assert!(StreamPayloadRound::matches(
            &mut r,
            &mut stream,
            &mut svc,
            &out,
            &mut w,
            &mut codec,
            &mut cap,
            "",
            "rtmp",
            1,
        )
        .is_err());
    }

    #[test]
    fn differential() {
        let mut ra = runner();
        let mut rb = runner();
        let mut sa = ProgramStream::new();
        let mut sb = ProgramStream::new();
        let mut sva = OutputService::new();
        let mut svb = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut wa = LiveWeights::new();
        let mut wb = LiveWeights::new();
        let mut ca = H264Codec::new(16, 16, &wa).unwrap();
        let mut cb = H264Codec::new(16, 16, &wb).unwrap();
        let mut capa = SyntheticCapture::new(16, 16);
        let mut capb = SyntheticCapture::new(16, 16);
        assert!(StreamPayloadRound::matches(
            &mut ra, &mut sa, &mut sva, &out, &mut wa, &mut ca, &mut capa, "twitch", "rtmp", 1,
        )
        .unwrap());
        assert!(StreamPayloadRound::matches(
            &mut rb, &mut sb, &mut svb, &out, &mut wb, &mut cb, &mut capb, "youtube", "rtmp", 2,
        )
        .unwrap());
    }
}
