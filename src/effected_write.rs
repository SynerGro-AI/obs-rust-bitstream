//! Safe pure EffectedWrite (effect, push, write core).
//!
//! Applies a SceneEffect, pushes into a sequence, and
//! writes the sequence. Reads LiveWeights. No unwraps.
//!
//! Differential: a bright step is in the written bytes.

use crate::ai::AiAssistant;
use crate::crossfade::Crossfade;
use crate::resample::Resample;
use crate::scene_effect::SceneEffect;
use crate::system_status::SystemStatus;
use crate::av_bundle::AvBundle;
use crate::audio_writer::AudioWriter;
use crate::audio_sync::AudioSync;
use crate::timestamp_stamp::TimestampStamp;
use crate::encode_cancel::EncodeCancel;
use crate::encode_status::EncodeStatus;
use crate::encode_progress::EncodeProgress;
use crate::encoder_controls::EncoderControls;
use crate::h264_codec::H264Codec;
use crate::fps_trailer::FpsTrailer;
use crate::timed_sequence::TimedSequence;
use crate::ping_pong::PingPong;
use crate::sequence_reverse::SequenceReverse;
use crate::frame_hold::FrameHold;
use crate::frame_sequence::FrameSequence;
use crate::live_weights::LiveWeights;
use crate::pipe_suggest::PipeSuggest;
use crate::raw_writer::{RawWriter, WriteError};
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

    /// Fade several steps across frames. Each step fades from 0 to its end.
    pub fn write_fade_steps<W: Write>(
        frames: &[VideoFrame],
        steps: &[(&str, f32)],
        out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut seq = FrameSequence::new();
        let mut pipe = ToolPipeline::new();
        for (name, _) in steps {
            pipe.add(name);
        }
        for (i, f) in frames.iter().enumerate() {
            let mut w = LiveWeights::new();
            for (name, end) in steps {
                let key = format!("pipe.{}", name);
                WeightInterp::set(&mut w, &key, i, frames.len(), 0.0, *end);
            }
            let mut processed = f.clone();
            let _ = pipe.run(&mut processed, &w);
            seq.push(processed);
        }
        RawWriter::write(&seq, out)
    }

    /// Crossfade from the first frame to the last across the sequence.
    pub fn write_crossfade<W: Write>(
        frames: &[VideoFrame],
        out: W,
    ) -> Result<(), WriteError> {
        if frames.len() < 2 {
            return Err(WriteError::Empty);
        }
        let first = &frames[0];
        let last = &frames[frames.len() - 1];
        let mut seq = FrameSequence::new();
        for i in 0..frames.len() {
            let t = i as f32 / (frames.len() - 1) as f32;
            let blended = Crossfade::blend(first, last, t).map_err(|_| WriteError::Empty)?;
            seq.push(blended);
        }
        RawWriter::write(&seq, out)
    }

    /// Hold a frame for count frames and write.
    pub fn write_hold<W: Write>(
        frame: &VideoFrame,
        count: usize,
        out: W,
    ) -> Result<(), WriteError> {
        if count == 0 {
            return Err(WriteError::Empty);
        }
        let mut seq = FrameSequence::new();
        FrameHold::hold(&mut seq, frame, count);
        RawWriter::write(&seq, out)
    }

    /// Reverse a sequence and write it.
    pub fn write_reverse<W: Write>(
        frames: &[VideoFrame],
        out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut seq = FrameSequence::new();
        for f in frames {
            seq.push(f.clone());
        }
        let rev = SequenceReverse::reverse(&seq);
        RawWriter::write(&rev, out)
    }

    /// Bounce a sequence forward then backward and write it.
    pub fn write_ping_pong<W: Write>(
        frames: &[VideoFrame],
        out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut seq = FrameSequence::new();
        for f in frames {
            seq.push(f.clone());
        }
        let bounced = PingPong::bounce(&seq);
        RawWriter::write(&bounced, out)
    }

    /// Write the frames from a timed sequence.
    pub fn write_timed<W: Write>(
        timed: &TimedSequence,
        out: W,
    ) -> Result<(), WriteError> {
        if timed.len() == 0 {
            return Err(WriteError::Empty);
        }
        RawWriter::write(timed.frames(), out)
    }

    /// Write the frames and append the fps trailer.
    pub fn write_timed_fps<W: Write>(
        timed: &TimedSequence,
        mut out: W,
    ) -> Result<(), WriteError> {
        if timed.len() == 0 {
            return Err(WriteError::Empty);
        }
        RawWriter::write(timed.frames(), &mut out)?;
        FpsTrailer::write(timed.fps(), &mut out)
    }

    /// Resample to a new fps and write with trailer.
    pub fn write_resampled<W: Write>(
        timed: &TimedSequence,
        new_fps: f32,
        out: W,
    ) -> Result<(), WriteError> {
        let resampled = Resample::to_fps(timed, new_fps).map_err(|_| WriteError::Empty)?;
        Self::write_timed_fps(&resampled, out)
    }

    /// Encode a sequence with the control panel and write Annex-B.
    pub fn write_h264<W: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        mut out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        for f in frames {
            let bytes = enc.encode(f).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Encode and report progress.
    pub fn write_h264_progress<W: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        progress: &mut EncodeProgress,
        mut out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        for f in frames {
            let bytes = enc.encode(f).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
            progress.tick();
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Apply a scene effect, then encode to H.264.
    pub fn write_effected_h264<W: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        out: W,
    ) -> Result<(), WriteError> {
        Self::write_effected_h264_progress(
            frames,
            effect,
            controls,
            weights,
            &mut EncodeProgress::new(frames.len()),
            out,
        )
    }

    /// Apply a scene effect, encode, and report progress.
    pub fn write_effected_h264_progress<W: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        progress: &mut EncodeProgress,
        mut out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        for f in frames {
            let mut processed = f.clone();
            let _ = effect.apply(&mut processed, weights);
            let bytes = enc.encode(&processed).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
            progress.tick();
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Encode and update an EncodeStatus.
    pub fn write_h264_status<W: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        status: &mut EncodeStatus,
        mut out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        for f in frames {
            let bytes = enc.encode(f).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
            status.tick();
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Encode and stop if cancel is set.
    pub fn write_h264_cancellable<W: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        status: &mut EncodeStatus,
        cancel: &EncodeCancel,
        mut out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        for f in frames {
            if cancel.is_cancelled() {
                break;
            }
            let bytes = enc.encode(f).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
            status.tick();
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Apply effect, encode, and stop if cancel is set.
    pub fn write_effected_h264_cancellable<W: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        status: &mut EncodeStatus,
        cancel: &EncodeCancel,
        mut out: W,
    ) -> Result<(), WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        for f in frames {
            if cancel.is_cancelled() {
                break;
            }
            let mut processed = f.clone();
            let _ = effect.apply(&mut processed, weights);
            let bytes = enc.encode(&processed).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
            status.tick();
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Encode and sample audio at each frame.
    pub fn write_h264_audio<W: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        mut out: W,
    ) -> Result<Vec<f32>, WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        controls.apply(weights).map_err(|_| WriteError::Empty)?;
        let first = &frames[0];
        let mut enc = H264Codec::new(first.width, first.height, weights)
            .map_err(|_| WriteError::Empty)?;
        let mut samples = Vec::new();
        for (i, f) in frames.iter().enumerate() {
            let bytes = enc.encode(f).map_err(|_| WriteError::Empty)?;
            out.write_all(&bytes).map_err(|_| WriteError::Io)?;
            samples.push(sync.sample(i));
        }
        let tail = enc.flush();
        out.write_all(&tail).map_err(|_| WriteError::Io)?;
        Ok(samples)
    }

    /// Encode video and write audio samples to a second writer.
    pub fn write_h264_av<W: Write, A: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        video: W,
        audio: A,
    ) -> Result<(), WriteError> {
        let samples = Self::write_h264_audio(frames, controls, weights, sync, video)?;
        AudioWriter::write(&samples, audio).map_err(|_| WriteError::Io)
    }

    /// Encode into an AvBundle.
    pub fn write_h264_bundle(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        bundle: &mut AvBundle,
    ) -> Result<(), WriteError> {
        let mut video = Vec::new();
        let samples = Self::write_h264_audio(frames, controls, weights, sync, &mut video)?;
        bundle.push_video(&video);
        bundle.push_audio(&samples);
        bundle.set_fps(sync.video().fps());
        bundle.set_encoder(crate::encoder_select::EncoderSelect::choose());
        Ok(())
    }

    /// Encode into a bundle and write both the bundle and the status.
    pub fn write_h264_bundle_status<W: Write, S: Write>(
        frames: &[VideoFrame],
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        bundle_out: W,
        status_out: S,
    ) -> Result<AvBundle, WriteError> {
        let mut bundle = AvBundle::new();
        Self::write_h264_bundle(frames, controls, weights, sync, &mut bundle)?;
        bundle.write(bundle_out).map_err(|_| WriteError::Io)?;
        let status = SystemStatus::from_bundle(&bundle);
        status.write(status_out).map_err(|_| WriteError::Io)?;
        Ok(bundle)
    }

    /// Apply an effect, encode, and write bundle and status.
    pub fn write_effected_bundle_status<W: Write, S: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        bundle_out: W,
        status_out: S,
    ) -> Result<AvBundle, WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut processed = Vec::new();
        for f in frames {
            let mut p = f.clone();
            let _ = effect.apply(&mut p, weights);
            processed.push(p);
        }
        Self::write_h264_bundle_status(
            &processed,
            controls,
            weights,
            sync,
            bundle_out,
            status_out,
        )
    }

    /// Apply an effect, encode with progress, and write both.
    pub fn write_effected_bundle_progress<W: Write, S: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        progress: &mut EncodeProgress,
        bundle_out: W,
        status_out: S,
    ) -> Result<AvBundle, WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut processed = Vec::new();
        for f in frames {
            let mut p = f.clone();
            let _ = effect.apply(&mut p, weights);
            processed.push(p);
            progress.tick();
        }
        Self::write_h264_bundle_status(
            &processed,
            controls,
            weights,
            sync,
            bundle_out,
            status_out,
        )
    }

    /// Apply an effect, encode with progress and cancel, and write both.
    pub fn write_effected_bundle_cancellable<W: Write, S: Write>(
        frames: &[VideoFrame],
        effect: &SceneEffect,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        progress: &mut EncodeProgress,
        cancel: &EncodeCancel,
        bundle_out: W,
        status_out: S,
    ) -> Result<AvBundle, WriteError> {
        if frames.is_empty() {
            return Err(WriteError::Empty);
        }
        let mut processed = Vec::new();
        for f in frames {
            if cancel.is_cancelled() {
                break;
            }
            let mut p = f.clone();
            let _ = effect.apply(&mut p, weights);
            processed.push(p);
            progress.tick();
        }
        if processed.is_empty() {
            return Err(WriteError::Empty);
        }
        Self::write_h264_bundle_status(
            &processed,
            controls,
            weights,
            sync,
            bundle_out,
            status_out,
        )
    }

    /// Stamp each frame with its time, then encode and write both.
    pub fn write_stamped_bundle<W: Write, S: Write>(
        frames: &[VideoFrame],
        fps: f32,
        controls: &EncoderControls,
        weights: &mut LiveWeights,
        sync: &AudioSync,
        bundle_out: W,
        status_out: S,
    ) -> Result<AvBundle, WriteError> {
        if frames.is_empty() || fps <= 0.0 {
            return Err(WriteError::Empty);
        }
        let mut stamped = Vec::new();
        for (i, f) in frames.iter().enumerate() {
            let mut p = f.clone();
            let secs = (i as f32 / fps) as u32;
            let _ = TimestampStamp::stamp_corner(&mut p, secs, 255);
            stamped.push(p);
        }
        Self::write_h264_bundle_status(
            &stamped,
            controls,
            weights,
            sync,
            bundle_out,
            status_out,
        )
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
        let mut buf6 = Vec::new();
        EffectedWrite::write_fade_steps(&[f.clone()], &[("bright", 10.0), ("glow", 5.0)], &mut buf6).unwrap();
        assert!(buf6.len() > 16);
        let mut dark = f.clone();
        dark.planes[0].fill(0);
        let mut bright = f.clone();
        bright.planes[0].fill(100);
        let mut buf7 = Vec::new();
        EffectedWrite::write_crossfade(&[dark, bright], &mut buf7).unwrap();
        assert!(buf7.len() > 16);
        let mut buf8 = Vec::new();
        EffectedWrite::write_hold(&f, 3, &mut buf8).unwrap();
        assert!(buf8.len() > 16);
        let mut buf9 = Vec::new();
        EffectedWrite::write_reverse(&[f.clone()], &mut buf9).unwrap();
        assert!(buf9.len() > 16);
        let mut buf10 = Vec::new();
        EffectedWrite::write_ping_pong(&[f.clone()], &mut buf10).unwrap();
        assert!(buf10.len() > 16);
        let mut timed = TimedSequence::new(30.0).unwrap();
        timed.push(f.clone());
        let mut buf11 = Vec::new();
        EffectedWrite::write_timed(&timed, &mut buf11).unwrap();
        assert!(buf11.len() > 16);
        let mut buf12 = Vec::new();
        EffectedWrite::write_timed_fps(&timed, &mut buf12).unwrap();
        let fps = FpsTrailer::read(&buf12).unwrap();
        assert_eq!(fps, 30.0);
        let mut buf13 = Vec::new();
        EffectedWrite::write_resampled(&timed, 15.0, &mut buf13).unwrap();
        let fps2 = FpsTrailer::read(&buf13).unwrap();
        assert_eq!(fps2, 15.0);
        let controls = EncoderControls::new().unwrap();
        let mut buf14 = Vec::new();
        EffectedWrite::write_h264(&[f.clone()], &controls, &mut w, &mut buf14).unwrap();
        assert!(!buf14.is_empty());
        let mut status = EncodeStatus::new(1, 30.0).unwrap();
        let mut buf16 = Vec::new();
        EffectedWrite::write_h264_status(&[f.clone()], &controls, &mut w, &mut status, &mut buf16).unwrap();
        assert!(status.is_done());
        let mut cancel = EncodeCancel::new();
        cancel.cancel();
        let mut status2 = EncodeStatus::new(2, 30.0).unwrap();
        let mut buf17 = Vec::new();
        EffectedWrite::write_h264_cancellable(&[f.clone(), f.clone()], &controls, &mut w, &mut status2, &cancel, &mut buf17).unwrap();
        assert_eq!(status2.done(), 0);
    }
}
