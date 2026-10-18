//! Safe pure StreamPayload (stream run + H.264 packets).
//!
//! Runs a StreamDump and a PayloadDump from the same capture
//! size. Returns the status buffer and the packet buffer.
//! No unwraps.
//!
//! Differential: more frames produce more packet bytes.

use crate::capture::Capture;
use crate::h264_codec::H264Codec;
use crate::live_weights::LiveWeights;
use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::payload_dump::{PayloadDump, PayloadError};
use crate::program_stream::ProgramStream;
use crate::runner::Runner;
use crate::stream_dump::StreamDump;
use crate::stream_run::StreamRunError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamPayloadError {
    Run(StreamRunError),
    Payload(PayloadError),
}

pub struct StreamPayload;

impl StreamPayload {
    /// Run the stream and encode the same number of frames.
    /// Returns (status_buf, packet_buf).
    pub fn run(
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
    ) -> Result<(Vec<u8>, Vec<u8>), StreamPayloadError> {
        let (_, status_buf) = StreamDump::run_and_write(
            runner, stream, service, output, weights, name, id, frames,
        )
        .map_err(StreamPayloadError::Run)?;
        let packet_buf =
            PayloadDump::write(codec, capture, frames).map_err(StreamPayloadError::Payload)?;
        Ok((status_buf, packet_buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::{AudioSource, AudioSourceList};
    use crate::capture::SyntheticCapture;
    use crate::output_info::{OutputInfo, OUTPUT_AV};
    use crate::pipeline::Pipeline;
    use crate::stream_file::StreamFile;

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
    fn writes_both() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let mut codec = H264Codec::new(16, 16, &w).unwrap();
        let mut cap = SyntheticCapture::new(16, 16);
        let (st, pk) = StreamPayload::run(
            &mut r, &mut stream, &mut svc, &out, &mut w, &mut codec, &mut cap,
            "twitch", "rtmp", 2,
        )
        .unwrap();
        assert_eq!(StreamFile::streaming(&st), Some(true));
        assert!(PayloadDump::header(&pk).unwrap().bytes > 0);
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
        assert!(StreamPayload::run(
            &mut r, &mut stream, &mut svc, &out, &mut w, &mut codec, &mut cap,
            "", "rtmp", 1,
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
        let (_, pa) = StreamPayload::run(
            &mut ra, &mut sa, &mut sva, &out, &mut wa, &mut ca, &mut capa,
            "twitch", "rtmp", 1,
        )
        .unwrap();
        let (_, pb) = StreamPayload::run(
            &mut rb, &mut sb, &mut svb, &out, &mut wb, &mut cb, &mut capb,
            "youtube", "rtmp", 3,
        )
        .unwrap();
        let ha = PayloadDump::header(&pa).unwrap();
        let hb = PayloadDump::header(&pb).unwrap();
        assert!(hb.bytes >= ha.bytes);
    }
}
