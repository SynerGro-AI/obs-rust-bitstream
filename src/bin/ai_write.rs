//! Bundle encoder demo.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::system_status::SystemStatus;
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
    let f = VideoFrame::new(VideoFormat::Y800, 16, 16).expect("frame");
    timed.push(f);
    let sync = AudioSync::new(timed.clone());
    let frames: Vec<_> = (0..timed.len())
        .filter_map(|i| timed.frames().get(i).ok().cloned())
        .collect();
    let mut controls = EncoderControls::new().expect("controls");
    let mut weights = LiveWeights::new();
    controls.set_quality(30.0, &mut weights).expect("quality");
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
        "bundle: video={} encoder={}",
        back.video_len(),
        back.encoder().as_str()
    );
    let status = SystemStatus::from_bundle(&back);
    println!("status: {}", status.summary());
}