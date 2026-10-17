//! Full path demo: effect then H.264.
//!
//! Builds four frames, applies a bright effect,
//! and writes out.h264.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::encoder_controls::EncoderControls;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::scene_effect::SceneEffect;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut frames = Vec::new();
    for i in 0..4 {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
        frame.planes[0].fill(40 + i * 20);
        frames.push(frame);
    }
    let mut effect = SceneEffect::new();
    effect.add("bright");
    let mut controls = EncoderControls::new().expect("controls");
    let mut weights = LiveWeights::new();
    weights.set("pipe.bright", 20.0).expect("weight");
    controls.set_quality(28.0, &mut weights).expect("quality");
    let file = File::create("out.h264").expect("file");
    EffectedWrite::write_effected_h264(&frames, &effect, &controls, &mut weights, file)
        .expect("write");
    println!("wrote out.h264 ({} frames, bright + H.264)", frames.len());
}