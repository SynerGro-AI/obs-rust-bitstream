//! Safe pure CaptureRate (frames per second from a source).
//!
//! Captures a few frames and returns the rate.
//! No unwraps.
//!
//! Differential: the rate is non-negative.

use crate::capture_select::CaptureSource;
use crate::write_timing::WriteTiming;

pub struct CaptureRate;

impl CaptureRate {
    /// Capture n frames and return fps.
    pub fn measure(src: &mut CaptureSource, n: usize) -> f32 {
        if n == 0 {
            return 0.0;
        }
        let timing = WriteTiming::start();
        let mut got = 0;
        for _ in 0..n {
            if src.frame().is_some() {
                got += 1;
            }
        }
        timing.fps(got)
    }

    /// True if capture is faster than encode.
    pub fn faster_than(capture_fps: f32, encode_fps: f32) -> bool {
        capture_fps > encode_fps
    }

    /// Which side is the bottleneck.
    pub fn bottleneck(capture_fps: f32, encode_fps: f32) -> &'static str {
        if capture_fps > encode_fps {
            "encode"
        } else {
            "capture"
        }
    }

    /// A short recommendation.
    pub fn recommend(capture_fps: f32, encode_fps: f32) -> &'static str {
        match Self::bottleneck(capture_fps, encode_fps) {
            "encode" => "lower quality or resolution",
            _ => "use a faster source",
        }
    }

    /// True if both rates are above a floor.
    pub fn healthy(capture_fps: f32, encode_fps: f32, floor: f32) -> bool {
        capture_fps >= floor && encode_fps >= floor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut src = CaptureSource::synthetic(8, 8);
        let rate = CaptureRate::measure(&mut src, 4);
        assert!(rate >= 0.0);
        assert!(CaptureRate::faster_than(100.0, 30.0));
        assert!(!CaptureRate::faster_than(10.0, 30.0));
        assert_eq!(CaptureRate::bottleneck(100.0, 30.0), "encode");
        assert_eq!(CaptureRate::bottleneck(10.0, 30.0), "capture");
        assert!(CaptureRate::recommend(100.0, 30.0).contains("quality"));
        assert!(CaptureRate::healthy(60.0, 30.0, 24.0));
        assert!(!CaptureRate::healthy(10.0, 30.0, 24.0));
    }
}
