//! Vertical slice binary.
//!
//! Loads one pure ColorSource, renders frames, and writes
//! them with RawWriter. No capture, no encoder, no UI.
//! Uses only library modules.

use obs_rust_bitstream::{
    ColorSource, FrameRenderer, FrameSequence, RawWriter,
};
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let color = ColorSource::new(255, 64, 0);
    let mut seq = FrameSequence::new();
    for _ in 0..30 {
        match FrameRenderer::solid(&color, 64, 48) {
            Ok(frame) => seq.push(frame),
            Err(_) => {
                eprintln!("render failed");
                std::process::exit(1);
            }
        }
    }
    let path = "slice.obsr";
    let file = match File::create(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("create failed");
            std::process::exit(1);
        }
    };
    let mut out = BufWriter::new(file);
    match RawWriter::write(&seq, &mut out) {
        Ok(()) => println!("wrote {} frames to {}", seq.len(), path),
        Err(_) => {
            eprintln!("write failed");
            std::process::exit(1);
        }
    }
}
