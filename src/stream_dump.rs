//! Safe pure StreamDump (run → status file).
//!
//! Runs a StreamRun and writes the StreamStatus.
//! Reads the streaming flag back. No unwraps.
//!
//! Differential: a running stream reads true.

use crate::live_weights::LiveWeights;
use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::program_stream::ProgramStream;
use crate::runner::Runner;
use crate::stream_file::StreamFile;
use crate::stream_run::{StreamRun, StreamRunError};
use crate::stream_status::StreamStatus;

pub struct StreamDump;

impl StreamDump {
    /// Run and write the status. Returns (status, bytes).
    pub fn run_and_write(
        runner: &mut Runner,
        stream: &mut ProgramStream,
        service: &mut OutputService,
        output: &OutputInfo,
        weights: &mut LiveWeights,
        name: &str,
        id: &str,
        frames: usize,
    ) -> Result<(StreamStatus, Vec<u8>), StreamRunError> {
        let status = StreamRun::run(runner, stream, service, output, weights, name, id, frames)?;
        let buf = StreamFile::write(&status);
        Ok((status, buf))
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
    fn writes() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let (st, buf) = StreamDump::run_and_write(
            &mut r,
            &mut stream,
            &mut svc,
            &out,
            &mut w,
            "twitch",
            "rtmp",
            1,
        )
        .unwrap();
        assert!(st.streaming);
        assert_eq!(StreamFile::streaming(&buf), Some(true));
    }

    #[test]
    fn empty_rejected() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        assert!(StreamDump::run_and_write(
            &mut r,
            &mut stream,
            &mut svc,
            &out,
            &mut w,
            "",
            "rtmp",
            1
        )
        .is_err());
    }

    #[test]
    fn differential() {
        let mut a = runner();
        let mut b = runner();
        let mut sa = ProgramStream::new();
        let mut sb = ProgramStream::new();
        let mut sva = OutputService::new();
        let mut svb = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut wa = LiveWeights::new();
        let mut wb = LiveWeights::new();
        let (_, ba) = StreamDump::run_and_write(
            &mut a, &mut sa, &mut sva, &out, &mut wa, "twitch", "rtmp", 1,
        )
        .unwrap();
        let (_, bb) = StreamDump::run_and_write(
            &mut b, &mut sb, &mut svb, &out, &mut wb, "youtube", "rtmp", 1,
        )
        .unwrap();
        assert!(StreamFile::streaming(&ba) == Some(true));
        assert!(StreamFile::streaming(&bb) == Some(true));
        assert_ne!(ba, bb);
    }
}
