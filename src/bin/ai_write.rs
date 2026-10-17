//! Bundle demo.
//!
//! Encodes into an AvBundle and writes out.avbn.
//! Reads it back and prints the lengths.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::av_bundle::AvBundle;
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
    let mut bundle = AvBundle::new();
    EffectedWrite::write_h264_bundle(
        &frames,
        &controls,
        &mut weights,
        &sync,
        &mut bundle,
    )
    .expect("encode");
    let file = File::create("out.avbn").expect("file");
    bundle.write(file).expect("write");
    let back = AvBundle::read(File::open("out.avbn").expect("open")).expect("read");
    println!(
        "bundle: video={} audio={} fps={} duration_ms={:.1}",
        back.video_len(),
        back.audio_len(),
        back.fps(),
        back.duration_ms()
    );
}