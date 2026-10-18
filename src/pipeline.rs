//! Safe pure Pipeline (end-to-end wire).
//!
//! Capture → optional tool suite (compositor / canvas / effects)
//! → H264Codec (LiveWeights) → AvTick → OutputInfo.
//! Frames stay OBS-shaped. No unwraps on the happy path.
//! No synthetic-only path. Result-only errors.
//!
//! Differential: a capture frame that survives the stack
//! produces more output bytes than an empty tick.

use crate::audio_source_list::AudioSourceList;
use crate::av_tick::AvTick;
use crate::capture::Capture;
use crate::compositor::Compositor;
use crate::h264_codec::H264Codec;
use crate::live_weights::LiveWeights;
use crate::output_info::{OutputInfo, OUTPUT_AV};
use crate::pixel_canvas::PixelCanvas;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineError {
    NoFrame,
    Encode,
    Size,
}

pub struct Pipeline {
    weights: LiveWeights,
    codec: H264Codec,
    canvas: PixelCanvas,
}

impl Pipeline {
    /// Build with size and weights. Codec and canvas match the capture size.
    pub fn new(width: u32, height: u32, weights: LiveWeights) -> Result<Self, PipelineError> {
        if width == 0 || height == 0 {
            return Err(PipelineError::Size);
        }
        let codec = H264Codec::new(width, height, &weights).map_err(|_| PipelineError::Encode)?;
        let canvas = PixelCanvas::new(width, height).map_err(|_| PipelineError::Size)?;
        Ok(Self {
            weights,
            codec,
            canvas,
        })
    }

    /// Run one tick: pull capture → optional paint → encode → AvTick.
    /// Returns the number of packets accepted by the output.
    pub fn tick(
        &mut self,
        capture: &mut dyn Capture,
        audio: &mut AudioSourceList,
        output: &mut OutputInfo,
        pts: i64,
    ) -> Result<usize, PipelineError> {
        let frame = capture.frame().ok_or(PipelineError::NoFrame)?;
        // Optional tool-suite pass: paint the real Y800 frame onto the canvas.
        // Later layers / effects / LiveWeights can mutate the canvas here.
        let _ = Compositor::paint_frame(&frame, &mut self.canvas);
        // Optional LiveWeights effect. Missing key leaves the canvas unchanged.
        let _ = crate::canvas_effect::CanvasEffect::apply(&mut self.canvas, &self.weights);
        // Encode the original OBS-shaped frame (canvas is the tool-suite side channel).
        let n = AvTick::tick(&mut self.codec, &frame, audio, output, pts);
        Ok(n)
    }

    pub fn weights(&self) -> &LiveWeights {
        &self.weights
    }

    pub fn canvas(&self) -> &PixelCanvas {
        &self.canvas
    }

    pub fn codec_mut(&mut self) -> &mut H264Codec {
        &mut self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;
    use crate::capture::SyntheticCapture;
    use crate::output_info::OutputInfo;

    fn setup() -> (Pipeline, SyntheticCapture, AudioSourceList, OutputInfo) {
        let mut weights = LiveWeights::new();
        let _ = weights.set("fps", 30.0);
        let _ = weights.set("h264.qp", 28.0);
        let pipe = Pipeline::new(16, 16, weights).expect("pipeline");
        let cap = SyntheticCapture::new(16, 16);
        let mut audio = AudioSourceList::new();
        let _ = audio.add("tone", AudioSource::synthetic(48000, 1));
        let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).expect("output");
        out.start();
        (pipe, cap, audio, out)
    }

    #[test]
    fn tick_feeds() {
        let (mut pipe, mut cap, mut audio, mut out) = setup();
        let n = pipe.tick(&mut cap, &mut audio, &mut out, 0).expect("tick");
        let _ = pipe.codec_mut().flush();
        assert!(n >= 1);
        assert!(out.bytes() > 0);
    }

    #[test]
    fn differential() {
        let (mut pipe, mut cap, mut audio, mut out) = setup();
        let before = out.bytes();
        let _ = pipe.tick(&mut cap, &mut audio, &mut out, 0);
        let _ = pipe.codec_mut().flush();
        assert!(out.bytes() > before);
    }

    #[test]
    fn no_frame_safe() {
        struct Empty;
        impl Capture for Empty {
            fn frame(&mut self) -> Option<VideoFrame> {
                None
            }
            fn width(&self) -> u32 {
                16
            }
            fn height(&self) -> u32 {
                16
            }
        }
        let (mut pipe, _, mut audio, mut out) = setup();
        let mut empty = Empty;
        assert_eq!(
            pipe.tick(&mut empty, &mut audio, &mut out, 0),
            Err(PipelineError::NoFrame)
        );
    }
}
