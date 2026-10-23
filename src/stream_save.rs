//! Safe pure StreamSave (buffers → files).
//!
//! Writes the status and packet buffers from a StreamPayload
//! to two paths. Returns the byte counts. No unwraps.
//!
//! Differential: a larger packet buffer produces a larger file.

use crate::capture::Capture;
use crate::h264_codec::H264Codec;
use crate::live_weights::LiveWeights;
use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::program_stream::ProgramStream;
use crate::runner::Runner;
use crate::stream_payload::{StreamPayload, StreamPayloadError};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamSaveError {
    Payload(StreamPayloadError),
    Io,
}

pub struct StreamSave;

impl StreamSave {
    /// Run and write status + packets to the two paths.
    /// Returns (status_bytes, packet_bytes).
    pub fn run_and_save(
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
        status_path: &Path,
        packet_path: &Path,
    ) -> Result<(usize, usize), StreamSaveError> {
        let (st, pk) = StreamPayload::run(
            runner, stream, service, output, weights, codec, capture, name, id, frames,
        )
        .map_err(StreamSaveError::Payload)?;
        fs::write(status_path, &st).map_err(|_| StreamSaveError::Io)?;
        fs::write(packet_path, &pk).map_err(|_| StreamSaveError::Io)?;
        Ok((st.len(), pk.len()))
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
    fn saves() {
        let dir = std::env::temp_dir();
        let sp = dir.join("stream_status_test.txt");
        let pp = dir.join("stream_packets_test.bin");
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let mut codec = H264Codec::new(16, 16, &w).unwrap();
        let mut cap = SyntheticCapture::new(16, 16);
        let (sb, pb) = StreamSave::run_and_save(
            &mut r,
            &mut stream,
            &mut svc,
            &out,
            &mut w,
            &mut codec,
            &mut cap,
            "twitch",
            "rtmp",
            1,
            &sp,
            &pp,
        )
        .unwrap();
        assert!(sb > 0);
        assert!(pb > 0);
        let status = fs::read(&sp).unwrap();
        assert_eq!(StreamFile::streaming(&status), Some(true));
        let _ = fs::remove_file(&sp);
        let _ = fs::remove_file(&pp);
    }

    #[test]
    fn empty_rejected() {
        let dir = std::env::temp_dir();
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let mut codec = H264Codec::new(16, 16, &w).unwrap();
        let mut cap = SyntheticCapture::new(16, 16);
        assert!(StreamSave::run_and_save(
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
            &dir.join("a"),
            &dir.join("b"),
        )
        .is_err());
    }

    #[test]
    fn differential() {
        let dir = std::env::temp_dir();
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
        let (_, pa) = StreamSave::run_and_save(
            &mut ra,
            &mut sa,
            &mut sva,
            &out,
            &mut wa,
            &mut ca,
            &mut capa,
            "twitch",
            "rtmp",
            1,
            &dir.join("sa"),
            &dir.join("pa"),
        )
        .unwrap();
        let (_, pb) = StreamSave::run_and_save(
            &mut rb,
            &mut sb,
            &mut svb,
            &out,
            &mut wb,
            &mut cb,
            &mut capb,
            "youtube",
            "rtmp",
            3,
            &dir.join("sb"),
            &dir.join("pb"),
        )
        .unwrap();
        assert!(pb >= pa);
        let _ = fs::remove_file(dir.join("sa"));
        let _ = fs::remove_file(dir.join("pa"));
        let _ = fs::remove_file(dir.join("sb"));
        let _ = fs::remove_file(dir.join("pb"));
    }
}
