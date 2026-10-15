//! Vertical slice binary.
//!
//! Builds a pure timeline, renders frames, applies a
//! brightness filter, and writes them with RawWriter.
//! No capture, no encoder, no UI. Uses only library modules.

use obs_rust_bitstream::{
    Draw, FrameFilter, PatternKind, PatternSource, RawWriter, SceneRenderer, SyntheticCamera,
    TextDraw, TextOverlay, VideoTimeline,
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
    let mut seq = match SceneRenderer::sequence(&timeline, 30, 1.0 / 15.0, 1920, 1080) {
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
    // Append frames from a synthetic camera so the file
    // contains time-varying content.
    if let Ok(cam) = SyntheticCamera::new(1920, 1080) {
        for i in 0..15 {
            let t = i as f32 / 15.0;
            if let Ok(mut f) = cam.frame(t) {
                let _ = FrameFilter::brightness(&mut f, 5);
                let _ = Draw::rect(&mut f, 100, 100, 200, 80, 255);
                let _ = TextDraw::text(&mut f, 110, 120, "HD", 0);
                let _ = TextOverlay::bordered(&mut f, 20, 20, "LIVE", 255, 200, 2);
                seq.push(f);
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
