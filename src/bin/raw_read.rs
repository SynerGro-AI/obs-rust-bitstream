//! Read a .obsr file and print its header and first pixel.
//!
//! Usage: cargo run --bin raw_read -- out.obsr

use obs_rust_bitstream::raw_reader::RawReader;
use std::env;
use std::fs::File;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "out.obsr".to_string());
    let file = File::open(&path).expect("open");
    let seq = RawReader::read(file).expect("read");
    println!("frames: {}", seq.len());
    for i in 0..seq.len() {
        if let Ok(f) = seq.get(i) {
            let first = f.planes.get(0).and_then(|p| p.first()).copied().unwrap_or(0);
            println!("frame {}: {}x{} first={}", i, f.width, f.height, first);
        }
    }
}
