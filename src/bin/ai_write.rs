//! Hold demo.
//!
//! Holds a mid-gray frame for four frames.
//! Writes out.obsr.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
    frame.planes[0].fill(128);
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_hold(&frame, 4, file).expect("write");
    println!("wrote out.obsr (hold 4)");
}