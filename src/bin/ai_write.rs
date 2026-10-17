//! Load a CC0 sample onto the timeline and sample it.

use obs_rust_bitstream::audio_timeline::AudioTimeline;
use obs_rust_bitstream::sound_cc0::SoundCc0;
use std::fs;

fn main() {
    let bytes = fs::read("/tmp/bell1.wav").expect("read");
    let data = SoundCc0::load(&bytes).expect("parse");
    let mut timeline = AudioTimeline::new();
    timeline.add("bell", 0.0).expect("add");
    timeline.load_wav("bell", &data).expect("load");
    println!(
        "bell1 on timeline: rate={} samples={}",
        data.sample_rate,
        data.samples.len()
    );
    for t in [0.0, 0.5, 1.0, 2.0] {
        println!("  {:.1}s -> {:.3}", t, timeline.render(t));
    }
}