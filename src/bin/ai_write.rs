//! AI write path demo.
//!
//! Builds a dark frame, lets AI set the weights,
//! runs bright/glow/sharpen, and writes out.obsr.

use obs_rust_bitstream::ai::{AiAssistant, AiConfig};
use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
    frame.planes[0].fill(40);
    let ai = AiAssistant::new(AiConfig::default());
    let mut weights = LiveWeights::new();
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_ai_steps(
        &[frame],
        &["bright", "glow", "sharpen"],
        &ai,
        &mut weights,
        file,
    )
    .expect("write");
    println!("wrote out.obsr");
}
