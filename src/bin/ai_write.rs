//! Combined write with CC0 audio.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::audio_timeline::AudioTimeline;
use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::encoder_controls::EncoderControls;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::rate_compare::RateCompare;
use obs_rust_bitstream::write_timing::WriteTiming;
use obs_rust_bitstream::encode_cancel::EncodeCancel;
use obs_rust_bitstream::encode_progress::EncodeProgress;
use obs_rust_bitstream::scene_effect::SceneEffect;
use obs_rust_bitstream::sound_cc0::SoundCc0;
use obs_rust_bitstream::system_status::SystemStatus;
use obs_rust_bitstream::timed_sequence::TimedSequence;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;

fn main() {
    let bytes = std::fs::read("/tmp/bell1.wav").expect("read");
    let data = SoundCc0::load(&bytes).expect("parse");
    let mut timeline = AudioTimeline::new();
    timeline.add("bell", 0.0).expect("add");
    timeline.load_wav("bell", &data).expect("load");
    let mut timed = TimedSequence::new(30.0).expect("fps");
    for _ in 0..8 {
        let f = VideoFrame::new(VideoFormat::Y800, 16, 16).expect("frame");
        timed.push(f);
    }
    let mut sync = AudioSync::new(timed.clone());
    *sync.audio_mut() = timeline;
    let frames: Vec<_> = (0..timed.len())
        .filter_map(|i| timed.frames().get(i).ok().cloned())
        .collect();
    let mut controls = EncoderControls::new().expect("controls");
    let mut weights = LiveWeights::new();
    controls.set_quality(30.0, &mut weights).expect("quality");
    weights.set("pipe.bright", 20.0).expect("bright");
    let mut effect = SceneEffect::new();
    effect.add("bright");
    let bundle_file = File::create("out.avbn").expect("bundle");
    let status_file = File::create("status.txt").expect("status");
    let timing = WriteTiming::start();
    let mut progress = EncodeProgress::new(frames.len());
    let cancel = EncodeCancel::new();
    let bundle = EffectedWrite::write_effected_bundle_cancellable(
        &frames,
        &effect,
        &controls,
        &mut weights,
        &sync,
        &mut progress,
        &cancel,
        bundle_file,
        status_file,
    )
    .expect("write");
    println!(
        "wrote out.avbn (video={} audio={}) progress={:.0}% time={:.1}ms fps={:.0}",
        bundle.video_len(),
        bundle.audio_len(),
        progress.fraction() * 100.0,
        timing.elapsed_ms(),
        timing.fps(frames.len())
    );
    let mut status = SystemStatus::from_bundle(&bundle);
    status.set_encode_ms(timing.elapsed_ms());
    println!("status: {}", status.summary());
    let status_file = File::create("status.txt").expect("status");
    status.write(status_file).expect("write status");
    println!("wrote status.txt with time");
    let (small, large) = RateCompare::fps(16, 32).expect("compare");
    println!("rate: 16x16={:.0} fps  32x32={:.0} fps", small, large);
    let (a, b, c) = RateCompare::three(8, 16, 32).expect("three");
    println!("three: 8={:.0} 16={:.0} 32={:.0}", a, b, c);
    println!("matches: {}", status.matches_bundle(&bundle));
}