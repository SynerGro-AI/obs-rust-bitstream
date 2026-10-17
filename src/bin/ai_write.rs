//! Combined write with CC0 audio.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::audio_timeline::AudioTimeline;
use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::encoder_controls::EncoderControls;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::capture_select::CaptureSource;
use obs_rust_bitstream::timestamp_stamp::TimestampStamp;
use obs_rust_bitstream::report::Report;
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
    let rate_file = File::create("rate.txt").expect("rate");
    RateCompare::write_three(8, 16, 32, rate_file).expect("write rate");
    println!("wrote rate.txt");
    let report_file = File::create("report.txt").expect("report");
    Report::write(&status, 8, 16, 32, report_file).expect("report");
    println!("wrote report.txt");
    let report_text = std::fs::read_to_string("report.txt").expect("read report");
    println!("report matches: {}", Report::matches_status(&report_text, &status));
    let mut stamped = frames[0].clone();
    TimestampStamp::stamp(&mut stamped, 0, 0, 60, 255).expect("stamp");
    println!("stamped frame with 00:01:00");
    TimestampStamp::stamp_corner(&mut stamped, 120, 200).expect("corner");
    println!("stamped corner with 00:02:00");
    println!("full: {}", TimestampStamp::format_full(2026, 10, 17, 60));
    println!("now: {}", TimestampStamp::now());
    println!("frame: {}", TimestampStamp::format_frame(60, 3));
    println!("progress: {}", TimestampStamp::format_progress(3, 10));
    println!("lum white: {}", TimestampStamp::lum("white"));
    weights.set("stamp.lum", 128.0).expect("lum");
    println!("stamp.lum set to 128");
    weights.set("stamp.opacity", 0.5).expect("opacity");
    println!("stamp.opacity set to 0.5 (blend={})", TimestampStamp::blend(128, 0.5));
    let mut src = CaptureSource::synthetic(16, 16);
    let mut captured_frames = Vec::new();
    for _ in 0..4 {
        if let Some(f) = src.frame() {
            captured_frames.push(f);
        }
    }
    println!("captured {} frames", captured_frames.len());
    let cap_bundle = File::create("captured.avbn").expect("cap");
    let cap_status = File::create("captured_status.txt").expect("status");
    let _ = EffectedWrite::write_stamped_bundle(
        &captured_frames,
        30.0,
        &controls,
        &mut weights,
        &sync,
        cap_bundle,
        cap_status,
    )
    .expect("encode captured");
    println!("wrote captured.avbn");
    let cap_size = std::fs::metadata("captured.avbn").map(|m| m.len()).unwrap_or(0);
    println!("captured.avbn size: {} bytes", cap_size);
    println!(
        "stamp example: {} {}",
        TimestampStamp::format_frame(0, 0),
        TimestampStamp::format_progress(0, frames.len())
    );
    println!("stamped encode uses frame index");
    let stamped_bundle = File::create("stamped.avbn").expect("stamped");
    let stamped_status = File::create("stamped_status.txt").expect("status");
    let _ = EffectedWrite::write_stamped_bundle(
        &frames,
        30.0,
        &controls,
        &mut weights,
        &sync,
        stamped_bundle,
        stamped_status,
    )
    .expect("stamped write");
    println!("wrote stamped.avbn");
    println!("base time: {}", TimestampStamp::now());
    println!("stamped: {}", TimestampStamp::is_stamped(&stamped, 0, 0, 48));
    status.set_stamped(true);
    status.set_stamp_opacity(0.5);
    println!("status with stamp: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with stamp flag");
    println!("matches: {}", status.matches_bundle(&bundle));
}