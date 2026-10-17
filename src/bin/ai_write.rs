//! Video + audio write demo.
//!
//! Encodes video and writes audio samples.
//! Writes out.h264 and out.raw.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::encoder_controls::EncoderControls;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::timed_sequence::TimedSequence;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let mut timed = TimedSequence::new(30.0).expect("fps");
    for i in 0..4 {
        let mut frame = VideoFrame::new(VideoFormat::Y800, 32, 32).expect("frame");
        frame.planes[0].fill(40 + i * 10);
        timed.push(frame);
    }
    let sync = AudioSync::new(timed.clone());
    let frames: Vec<_> = (0..timed.len())
        .filter_map(|i| timed.frames().get(i).ok().cloned())
        .collect();
    let mut controls = EncoderControls::new().expect("controls");
    let mut weights = LiveWeights::new();
    controls.set_quality(28.0, &mut weights).expect("quality");
    let video = File::create("out.h264").expect("video");
    let audio = File::create("out.raw").expect("audio");
    EffectedWrite::write_h264_av(
        &frames,
        &controls,
        &mut weights,
        &sync,
        video,
        audio,
    )
    .expect("write");
    println!("wrote out.h264 and out.raw ({} frames)", frames.len());
}