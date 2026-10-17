//! Multi-step fade demo.
//!
//! Builds four frames and fades bright and glow.
//! Writes out.obsr.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut frames = Vec::new();
    for _ in 0..4 {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
        frame.planes[0].fill(40);
        frames.push(frame);
    }
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_fade_steps(
        &frames,
        &[("bright", 20.0), ("glow", 10.0)],
        file,
    )
    .expect("write");
    println!("wrote out.obsr ({} frames, bright+glow fade)", frames.len());
}