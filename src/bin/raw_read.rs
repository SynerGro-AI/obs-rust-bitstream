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
    if let Ok(f) = seq.get(0) {
        println!("size: {}x{}", f.width, f.height);
        if let Some(p) = f.planes.get(0) {
            println!("first pixel: {}", p.first().copied().unwrap_or(0));
            let min = p.iter().copied().min().unwrap_or(0);
            let max = p.iter().copied().max().unwrap_or(0);
            println!("min/max: {}/{}", min, max);
        }
    }
}
