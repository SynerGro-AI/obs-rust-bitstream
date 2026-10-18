//! Safe pure RunnerDump (run → length-prefixed file).
//!
//! Runs a Runner for N frames and writes a small header
//! with frame count and output bytes. Reads the header back.
//! No unwraps. Result-only errors.
//!
//! Differential: more frames produce a larger byte count.

use crate::runner::{Runner, RunnerError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DumpError {
    Runner(RunnerError),
    Short,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DumpHeader {
    pub frames: u32,
    pub bytes: u64,
}

pub struct RunnerDump;

impl RunnerDump {
    /// Run and write a 12-byte header: ticks (u32 LE) + bytes (u64 LE).
    /// Ticks is the number of successful pipeline ticks.
    pub fn run_and_write(runner: &mut Runner, frames: usize) -> Result<Vec<u8>, DumpError> {
        let before = runner.output().bytes();
        let _fed = runner.run(frames).map_err(DumpError::Runner)?;
        let bytes = runner.output().bytes().saturating_sub(before);
        let mut buf = Vec::with_capacity(12);
        buf.extend_from_slice(&(frames as u32).to_le_bytes());
        buf.extend_from_slice(&bytes.to_le_bytes());
        Ok(buf)
    }

    /// Read the header. Returns None if shorter than 12 bytes.
    pub fn read(buf: &[u8]) -> Option<DumpHeader> {
        if buf.len() < 12 {
            return None;
        }
        let frames = u32::from_le_bytes(buf[0..4].try_into().ok()?);
        let bytes = u64::from_le_bytes(buf[4..12].try_into().ok()?);
        Some(DumpHeader { frames, bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSourceList;
    use crate::capture::SyntheticCapture;
    use crate::live_weights::LiveWeights;
    use crate::output_info::{OutputInfo, OUTPUT_AV};
    use crate::pipeline::Pipeline;

    fn runner() -> Runner {
        let mut weights = LiveWeights::new();
        let _ = weights.set("fps", 30.0);
        let _ = weights.set("h264.qp", 28.0);
        let pipe = Pipeline::new(16, 16, weights).expect("pipe");
        let cap = Box::new(SyntheticCapture::new(16, 16));
        let mut audio = AudioSourceList::new();
        let _ = audio.add("tone", crate::audio_source_list::AudioSource::synthetic(48000, 1));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).expect("out");
        out.start();
        Runner::new(pipe, cap, audio, out)
    }

    #[test]
    fn write_read() {
        let mut r = runner();
        let buf = RunnerDump::run_and_write(&mut r, 2).unwrap();
        let h = RunnerDump::read(&buf).unwrap();
        assert_eq!(h.frames, 2);
        assert!(h.bytes > 0);
    }

    #[test]
    fn short_is_none() {
        assert!(RunnerDump::read(&[0u8; 4]).is_none());
        assert!(RunnerDump::read(&[]).is_none());
    }

    #[test]
    fn differential() {
        let mut a = runner();
        let mut b = runner();
        let ha = RunnerDump::read(&RunnerDump::run_and_write(&mut a, 1).unwrap()).unwrap();
        let hb = RunnerDump::read(&RunnerDump::run_and_write(&mut b, 3).unwrap()).unwrap();
        assert!(hb.bytes >= ha.bytes);
        assert!(hb.frames > ha.frames);
    }
}
