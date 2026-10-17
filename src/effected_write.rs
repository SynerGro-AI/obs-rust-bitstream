//! Safe pure EffectedWrite (effect, push, write core).
//!
//! Applies a SceneEffect, pushes into a sequence, and
//! writes the sequence. Reads LiveWeights. No unwraps.
//!
//! Differential: a bright step is in the written bytes.

use crate::frame_sequence::FrameSequence;
use crate::live_weights::LiveWeights;
use crate::raw_writer::{RawWriter, WriteError};
use crate::scene_effect::SceneEffect;
use crate::tool_pipeline::ToolPipeline;
use crate::video_frame::VideoFrame;
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
        EffectedWrite::write(&[f], &effect, &w, &mut buf).unwrap();
        // header is 16 bytes, then pixels
        assert!(buf.len() > 16);
        assert_eq!(buf[16], 60);
        let mut seq = FrameSequence::new();
        seq.push(f);
        let pipe = ToolPipeline::new();
        let mut buf2 = Vec::new();
        EffectedWrite::write_sequence(&mut seq, &pipe, &w, &mut buf2).unwrap();
        assert!(buf2.len() > 16);
    }
}
