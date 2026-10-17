//! Controls panel demo.
//!
//! Sets quality and bitrate, prints the summary,
//! encodes, then resets.

use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::encoder_controls::EncoderControls;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut frames = Vec::new();
    let mut frame = VideoFrame::new(VideoFormat::Y800, 32, 32).expect("frame");
    frame.planes[0].fill(80);
    frames.push(frame);
    let mut controls = EncoderControls::new().expect("controls");
    let mut weights = LiveWeights::new();
    controls.set_quality(32.0, &mut weights).expect("quality");
    controls.set_bitrate(500_000.0, &mut weights).expect("bitrate");
    println!("{}", controls.summary());
    let file = File::create("out.h264").expect("file");
    EffectedWrite::write_h264(&frames, &controls, &mut weights, file).expect("write");
    controls.reset(&mut weights).expect("reset");
    println!("after reset: {}", controls.summary());
}