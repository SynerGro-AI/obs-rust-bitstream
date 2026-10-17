//! Crossfade demo.
//!
//! Builds a dark frame and a bright frame and
//! crossfades across four frames. Writes out.obsr.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut dark = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
    dark.planes[0].fill(0);
    let mut bright = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
    bright.planes[0].fill(200);
    let frames = vec![dark, bright.clone(), bright.clone(), bright];
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_crossfade(&frames, file).expect("write");
    println!("wrote out.obsr (crossfade 0..200)");
}