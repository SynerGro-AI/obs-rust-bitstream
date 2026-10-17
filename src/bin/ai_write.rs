//! Load a CC0 sample and fill encode samples.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::audio_timeline::AudioTimeline;
use obs_rust_bitstream::sound_cc0::SoundCc0;
use obs_rust_bitstream::timed_sequence::TimedSequence;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs;

fn main() {
    let bytes = fs::read("/tmp/bell1.wav").expect("read");
    let data = SoundCc0::load(&bytes).expect("parse");
    let mut timeline = AudioTimeline::new();
    timeline.add("bell", 0.0).expect("add");
    timeline.load_wav("bell", &data).expect("load");
    let mut timed = TimedSequence::new(30.0).expect("fps");
    for _ in 0..4 {
        let f = VideoFrame::new(VideoFormat::Y800, 16, 16).expect("frame");
        timed.push(f);
    }
    let mut sync = AudioSync::new(timed);
    *sync.audio_mut() = timeline;
    let samples = sync.fill(0, 4);
    println!("filled {} samples from CC0", samples.len());
    for (i, s) in samples.iter().enumerate() {
        println!("  frame {} -> {:.4}", i, s);
    }
}