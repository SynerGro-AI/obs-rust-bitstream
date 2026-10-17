//! Timed write demo.
//!
//! Builds a timed sequence at 24 fps and writes it
//! with an fps trailer. Writes out.obsr.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::timed_sequence::TimedSequence;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut timed = TimedSequence::new(24.0).expect("fps");
    for i in 0..4 {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
        frame.planes[0].fill(40 + i * 20);
        timed.push(frame);
    }
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_timed_fps(&timed, file).expect("write");
    println!("wrote out.obsr ({} frames at {} fps)", timed.len(), timed.fps());
}