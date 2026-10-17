//! Ping-pong demo.
//!
//! Builds three frames that brighten and writes them
//! forward then backward. Writes out.obsr.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut frames = Vec::new();
    for i in 0..3 {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
        frame.planes[0].fill(40 + i * 40);
        frames.push(frame);
    }
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_ping_pong(&frames, file).expect("write");
    println!("wrote out.obsr (ping-pong)");
}