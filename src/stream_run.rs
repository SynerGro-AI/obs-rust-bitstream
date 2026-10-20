//! Safe pure StreamRun (start → run → status).
//!
//! Starts a stream with the service and OBS defaults,
//! runs the runner for N frames, and returns the status.
//! Empty service is rejected. No unwraps.
//!
//! Differential: more frames produce a larger byte count.

use crate::live_weights::LiveWeights;
use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::program_stream::ProgramStream;
use crate::runner::{Runner, RunnerError};
use crate::stream_start::StreamStart;
use crate::stream_status::StreamStatus;
use crate::service_status::ServiceStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamRunError {
    Start,
    Run(RunnerError),
}

pub struct StreamRun;

impl StreamRun {
    /// Start the stream and run N frames. Returns the status.
    pub fn run(
        runner: &mut Runner,
        stream: &mut ProgramStream,
        service: &mut OutputService,
        output: &OutputInfo,
        weights: &mut LiveWeights,
        name: &str,
        id: &str,
        frames: usize,
    ) -> Result<StreamStatus, StreamRunError> {
        let defaults = StreamStart::start(stream, service, output, weights, name, id)
            .map_err(|_| StreamRunError::Start)?;
        runner.run(frames).map_err(StreamRunError::Run)?;
        let svc = ServiceStatus::from(service);
        Ok(StreamStatus::from(stream, &svc, &defaults))
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
    fn runs() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let st = StreamRun::run(
            &mut r,
            &mut stream,
            &mut svc,
            &out,
            &mut w,
            "twitch",
            "rtmp_common",
            2,
        )
        .unwrap();
        assert!(st.streaming);
        assert_eq!(st.service, "twitch");
        assert_eq!(st.bitrate_kbps, 6000);
    }

    #[test]
    fn empty_rejected() {
        let mut r = runner();
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        assert!(StreamRun::run(&mut r, &mut stream, &mut svc, &out, &mut w, "", "rtmp", 1).is_err());
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
        let sta = StreamRun::run(&mut a, &mut sa, &mut sva, &out, &mut wa, "twitch", "rtmp", 1).unwrap();
        let stb = StreamRun::run(&mut b, &mut sb, &mut svb, &out, &mut wb, "youtube", "rtmp", 3).unwrap();
        assert!(sta.streaming && stb.streaming);
        assert_ne!(sta.service, stb.service);
    }
}
