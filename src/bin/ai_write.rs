//! Load a CC0 sample and print its stats.

use obs_rust_bitstream::sound_cc0::SoundCc0;
use std::fs;

fn main() {
    let bytes = fs::read("/tmp/bell1.wav").expect("read");
    let data = SoundCc0::load(&bytes).expect("parse");
    println!(
        "bell1.wav: rate={} ch={} samples={} duration_s={:.2}",
        data.sample_rate,
        data.channels,
        data.samples.len(),
        data.samples.len() as f32 / data.sample_rate as f32
    );
    if let Some(peak) = data.samples.iter().map(|s| s.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()) {
        println!("peak: {:.3}", peak);
    }
}