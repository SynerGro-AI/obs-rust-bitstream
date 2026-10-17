//! Resample demo.
//!
//! Builds a timed sequence at 30 fps and writes it
//! resampled to 15 fps. Writes out.obsr.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::timed_sequence::TimedSequence;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut timed = TimedSequence::new(30.0).expect("fps");
    for i in 0..30 {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 64, 64).expect("frame");
        frame.planes[0].fill(i as u8 * 4);
        timed.push(frame);
    }
    let file = File::create("out.obsr").expect("file");
    EffectedWrite::write_resampled(&timed, 15.0, file).expect("write");
    println!("wrote out.obsr (30 fps -> 15 fps)");
}