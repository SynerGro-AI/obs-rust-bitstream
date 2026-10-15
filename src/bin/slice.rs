//! Vertical slice binary.
//!
//! Builds a pure timeline, renders frames, applies a
//! brightness filter, and writes them with RawWriter.
//! No capture, no encoder, no UI. Uses only library modules.

use obs_rust_bitstream::{
    FrameFilter, PatternKind, PatternSource, RawWriter, SceneRenderer, VideoTimeline,
};
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let mut timeline = VideoTimeline::new();
    if timeline.add("intro", 0.0, 1.0, "cam1").is_err()
        || timeline.add("main", 1.0, 1.0, "cam2").is_err()
    {
        eprintln!("timeline failed");
        std::process::exit(1);
    }
    let mut seq = match SceneRenderer::sequence(&timeline, 30, 1.0 / 15.0, 64, 48) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("render failed");
            std::process::exit(1);
        }
    };
    // Apply a pure brightness lift to every frame.
    for frame in seq.frames_mut() {
        let _ = FrameFilter::brightness(frame, 20);
    }
    // Also render one pattern frame so the file contains bars.
    if let Ok(pattern) = PatternSource::new(PatternKind::Bars, 64, 48) {
        if let Ok(mut pf) = obs_rust_bitstream::FrameRenderer::pattern(&pattern) {
            let _ = FrameFilter::brightness(&mut pf, 10);
            seq.push(pf);
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
