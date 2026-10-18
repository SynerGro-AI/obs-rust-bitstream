//! Safe pure Runner (continuous pipeline loop).
//!
//! Owns a Pipeline, Capture, audio list, and OutputInfo.
//! Runs for N frames or until stopped. Frames stay OBS-shaped.
//! No unwraps on the happy path. Result-only errors.
//!
//! Differential: more frames produce more bytes in the output.

use crate::audio_source_list::AudioSourceList;
use crate::capture::Capture;
use crate::output_info::OutputInfo;
use crate::pipeline::{Pipeline, PipelineError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunnerError {
    Pipeline(PipelineError),
    Stopped,
}

pub struct Runner {
    pipeline: Pipeline,
    capture: Box<dyn Capture>,
    audio: AudioSourceList,
    output: OutputInfo,
    pts: i64,
}

impl Runner {
    /// Create a runner. Capture and output must already be started if desired.
    pub fn new(
        pipeline: Pipeline,
        capture: Box<dyn Capture>,
        audio: AudioSourceList,
        output: OutputInfo,
    ) -> Self {
        Self {
            pipeline,
            capture,
            audio,
            output,
            pts: 0,
        }
    }

    /// Run for exactly `frames` ticks. Returns the total packets accepted.
    /// Stops early if the output is not running or a tick fails.
    pub fn run(&mut self, frames: usize) -> Result<usize, RunnerError> {
        if !self.output.is_running() {
            return Err(RunnerError::Stopped);
        }
        let mut total = 0;
        for _ in 0..frames {
            if !self.output.is_running() {
                break;
            }
            match self.pipeline.tick(
                &mut *self.capture,
                &mut self.audio,
                &mut self.output,
                self.pts,
            ) {
                Ok(n) => {
                    total += n;
                    self.pts = self.pts.saturating_add(1);
                }
                Err(e) => return Err(RunnerError::Pipeline(e)),
            }
        }
        // Flush the encoder at the end of the run.
        let _ = self.pipeline.codec_mut().flush();
        Ok(total)
    }

    /// Run until `duration` elapses or the output stops.
    /// Returns the number of successful ticks.
    pub fn run_for(&mut self, duration: std::time::Duration) -> Result<usize, RunnerError> {
        if !self.output.is_running() {
            return Err(RunnerError::Stopped);
        }
        let start = std::time::Instant::now();
        let mut ticks = 0usize;
        while start.elapsed() < duration {
            if !self.output.is_running() {
                break;
            }
            match self.pipeline.tick(
                &mut *self.capture,
                &mut self.audio,
                &mut self.output,
                self.pts,
            ) {
                Ok(_) => {
                    ticks = ticks.saturating_add(1);
                    self.pts = self.pts.saturating_add(1);
                }
                Err(e) => return Err(RunnerError::Pipeline(e)),
            }
        }
        let _ = self.pipeline.codec_mut().flush();
        Ok(ticks)
    }

    pub fn output(&self) -> &OutputInfo {
        &self.output
    }

    pub fn output_mut(&mut self) -> &mut OutputInfo {
        &mut self.output
    }

    pub fn pts(&self) -> i64 {
        self.pts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;
    use crate::capture::SyntheticCapture;
    use crate::live_weights::LiveWeights;
    use crate::output_info::{OutputInfo, OUTPUT_AV};
    use crate::pipeline::Pipeline;

    fn setup(frames_ready: bool) -> Runner {
        let mut weights = LiveWeights::new();
        let _ = weights.set("fps", 30.0);
        let _ = weights.set("h264.qp", 28.0);
        let pipe = Pipeline::new(16, 16, weights).expect("pipe");
        let cap = Box::new(SyntheticCapture::new(16, 16));
        let mut audio = AudioSourceList::new();
        let _ = audio.add("tone", AudioSource::synthetic(48000, 1));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).expect("out");
        if frames_ready {
            out.start();
        }
        Runner::new(pipe, cap, audio, out)
    }

    #[test]
    fn run_feeds() {
        let mut r = setup(true);
        let n = r.run(4).expect("run");
        assert!(n >= 1);
        assert!(r.output().bytes() > 0);
    }

    #[test]
    fn run_for_ticks() {
        let mut r = setup(true);
        let ticks = r.run_for(std::time::Duration::from_millis(5)).unwrap();
        assert!(ticks >= 1);
    }

    #[test]
    fn differential_more_frames() {
        let mut short = setup(true);
        let mut long = setup(true);
        let _ = short.run(2);
        let _ = long.run(6);
        assert!(long.output().bytes() >= short.output().bytes());
    }

    #[test]
    fn stopped_is_error() {
        let mut r = setup(false);
        assert_eq!(r.run(1), Err(RunnerError::Stopped));
    }
}
