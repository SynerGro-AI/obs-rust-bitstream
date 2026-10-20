//! Safe pure media timing helpers.
//!
//! Ties sample rate and frame rate into nanosecond durations.
//! Zero rates return None. No unwraps.
//!
//! Differential: durations match the mathematical 1e9 / rate.

#[cfg(test)]
use crate::diff::assert_equal;

/// Nanoseconds per audio sample at the given sample rate.
pub fn sample_duration_ns(sample_rate: u32) -> Option<u64> {
    if sample_rate == 0 {
        None
    } else {
        Some(1_000_000_000u64 / sample_rate as u64)
    }
}

/// Nanoseconds per video frame from a rational FPS.
pub fn frame_duration_ns(numerator: u32, denominator: u32) -> Option<u64> {
    if numerator == 0 || denominator == 0 {
        None
    } else {
        Some(1_000_000_000u64 * denominator as u64 / numerator as u64)
    }
}

/// Convert frames to samples (rounded toward zero).
pub fn frames_to_samples(frames: u32, fps_num: u32, fps_den: u32, sample_rate: u32) -> Option<u64> {
    if fps_num == 0 || fps_den == 0 || sample_rate == 0 || frames == 0 {
        return None;
    }
    let duration_ns = frame_duration_ns(fps_num, fps_den)? * frames as u64;
    Some(duration_ns * sample_rate as u64 / 1_000_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_48k() {
        let d = sample_duration_ns(48000).unwrap();
        assert_eq!(d, 1_000_000_000 / 48000);
    }

    #[test]
    fn frame_30() {
        let d = frame_duration_ns(30, 1).unwrap();
        assert_eq!(d, 1_000_000_000 / 30);
    }

    #[test]
    fn zero_safe() {
        assert!(sample_duration_ns(0).is_none());
        assert!(frame_duration_ns(0, 1).is_none());
        assert!(frames_to_samples(1, 30, 1, 0).is_none());
    }

    #[test]
    fn differential() {
        let d = sample_duration_ns(44100).unwrap();
        assert_equal(d, 1_000_000_000 / 44100, "sample ns");
    }
}
