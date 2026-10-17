//! Read a .obsr file and print frames, fps, and duration.
//!
//! Usage: cargo run --bin raw_read -- out.obsr

use obs_rust_bitstream::raw_reader::RawReader;
use std::env;
use std::fs::File;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "out.obsr".to_string());
    let file = File::open(&path).expect("open");
    let timed = RawReader::read_timed(file).expect("read");
    println!("frames: {}", timed.len());
    println!("fps: {}", timed.fps());
    println!("duration_ms: {:.1}", timed.duration_ms());
    if let Ok(f) = timed.frames().get(0) {
        println!("size: {}x{}", f.width, f.height);
    }
}