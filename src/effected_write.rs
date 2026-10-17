//! Safe pure EffectedWrite (effect, push, write core).
//!
//! Applies a SceneEffect, pushes into a sequence, and
//! writes the sequence. Reads LiveWeights. No unwraps.
//!
//! Differential: a bright step is in the written bytes.

use crate::ai::AiAssistant;
use crate::frame_sequence::FrameSequence;
use crate::live_weights::LiveWeights;
use crate::pipe_suggest::PipeSuggest;
use crate::raw_writer::{RawWriter, WriteError};
use crate::scene_effect::SceneEffect;
use crate::tool_pipeline::ToolPipeline;
use crate::video_frame::VideoFrame;
use crate::weight_interp::WeightInterp;
use std::io::Write;

pub struct EffectedWrite;

impl EffectedWrite {
    /// Apply the effect to each frame, then write the sequence.
    pub fn write<W: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        weights: &LiveWeights,
        out: W,
    ) -> Result<(), WriteError> {
        let mut seq = FrameSequence::new();
        for f in frames {
            let mut processed = f.clone();
            let _ = effect.apply(&mut processed, weights);
            seq.push(processed);
        }
        RawWriter::write(&seq, out)
    }

    /// Run a pipeline on a sequence and write it.
    pub fn write_sequence<W: Write>(
        seq: &mut FrameSequence,
        pipe: &ToolPipeline,
        weights: &LiveWeights,
        out: W,
    ) -> Result<(), WriteError> {
        pipe.run_sequence(seq, weights).map_err(|_| WriteError::Empty)?;
        RawWriter::write(seq, out)
    }

    /// AI suggests weights, chosen steps run, sequence is written.
    pub fn write_ai_steps<W: Write>(
        frames: &[VideoFrame],
        steps: &[&str],
        ai: &AiAssistant,
        weights: &mut LiveWeights,
        out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        PipeSuggest::full(ai, &frames[0], weights);
        let mut pipe = ToolPipeline::new();
        for s in steps {
            pipe.add(s);
        }
        let mut seq = FrameSequence::new();
        for f in frames {
            seq.push(f.clone());
        }
        pipe.run_sequence(&mut seq, weights).map_err(|_| WriteError::Empty)?;
        RawWriter::write(&seq, out)
    }

    /// AI suggests weights, pipeline runs, sequence is written.
    pub fn write_ai<W: Write>(
        frames: &[VideoFrame],
        ai: &AiAssistant,
        weights: &mut LiveWeights,
        out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        PipeSuggest::full(ai, &frames[0], weights);
        let mut pipe = ToolPipeline::new();
        pipe.add("bright");
        pipe.add("glow");
        pipe.add("sharpen");
        let mut seq = FrameSequence::new();
        for f in frames {
            seq.push(f.clone());
        }
        pipe.run_sequence(&mut seq, weights).map_err(|_| WriteError::Empty)?;
        RawWriter::write(&seq, out)
    }

    /// Write a sequence with a weight that fades across frames.
    pub fn write_fade<W: Write>(
        frames: &[VideoFrame],
        step: &str,
        start: f32,
        end: f32,
        out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut seq = FrameSequence::new();
        let mut pipe = ToolPipeline::new();
        pipe.add(step);
        let key = format!("pipe.{}", step);
        for (i, f) in frames.iter().enumerate() {
            let mut w = LiveWeights::new();
            WeightInterp::set(&mut w, &key, i, frames.len(), start, end);
            let mut processed = f.clone();
            let _ = pipe.run(&mut processed, &w);
            seq.push(processed);
        }
        RawWriter::write(&seq, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut effect = SceneEffect::new();
        effect.add("bright");
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(40);
        let mut w = LiveWeights::new();
        w.set("pipe.bright", 20.0).unwrap();
        let mut buf = Vec::new();
        EffectedWrite::write(&[f.clone()], &effect, &w, &mut buf).unwrap();
        // header is 16 bytes, then pixels
        assert!(buf.len() > 16);
        assert_eq!(buf[16], 60);
        let mut seq = FrameSequence::new();
        seq.push(f.clone());
        let pipe = ToolPipeline::new();
        let mut buf2 = Vec::new();
        EffectedWrite::write_sequence(&mut seq, &pipe, &w, &mut buf2).unwrap();
        assert!(buf2.len() > 16);
        let ai = crate::ai::AiAssistant::new(crate::ai::AiConfig::default());
        let mut buf3 = Vec::new();
        EffectedWrite::write_ai(&[f.clone()], &ai, &mut w, &mut buf3).unwrap();
        assert!(buf3.len() > 16);
        let mut buf4 = Vec::new();
        EffectedWrite::write_ai_steps(&[f.clone()], &["bright", "invert"], &ai, &mut w, &mut buf4).unwrap();
        assert!(buf4.len() > 16);
        let mut buf5 = Vec::new();
        EffectedWrite::write_fade(&[f.clone(), f.clone()], "bright", 0.0, 20.0, &mut buf5).unwrap();
        assert!(buf5.len() > 16);
    }
}
