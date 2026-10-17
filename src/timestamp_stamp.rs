//! Safe pure TimestampStamp (burn time into a frame).
//!
//! Formats seconds as HH:MM:SS and draws it onto
//! a frame. No unwraps.
//!
//! Differential: a stamp changes pixels.

use crate::draw::DrawError;
use crate::text_draw::TextDraw;
use crate::video_frame::VideoFrame;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TimestampStamp;

impl TimestampStamp {
    /// Format seconds as HH:MM:SS.
    pub fn format(secs: u32) -> String {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    /// Format a date and time as YYYY-MM-DD HH:MM:SS.
    pub fn format_full(year: u32, month: u32, day: u32, secs: u32) -> String {
        format!(
            "{:04}-{:02}-{:02} {}",
            year,
            month,
            day,
            Self::format(secs)
        )
    }

    /// Format a frame index and a total.
    pub fn format_progress(frame: usize, total: usize) -> String {
        format!("{}/{}", frame, total)
    }

    /// Named luminance levels.
    pub fn lum(name: &str) -> u8 {
        match name {
            "black" => 0,
            "dim" => 64,
            "mid" => 128,
            "bright" => 192,
            "white" => 255,
            _ => 255,
        }
    }

    /// Format seconds and a frame index.
    pub fn format_frame(secs: u32, frame: usize) -> String {
        format!("{} f{}", Self::format(secs), frame)
    }

    /// Current Unix time as HH:MM:SS (UTC).
    pub fn now() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);
        let day_secs = secs % 86400;
        Self::format(day_secs)
    }

    /// Draw a timestamp at (x, y).
    pub fn stamp(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        secs: u32,
        lum: u8,
    ) -> Result<u32, DrawError> {
        let text = Self::format(secs);
        TextDraw::text(frame, x, y, &text, lum)
    }

    /// Stamp at the bottom-right corner.
    pub fn stamp_corner(
        frame: &mut VideoFrame,
        secs: u32,
        lum: u8,
    ) -> Result<u32, DrawError> {
        let text = Self::format(secs);
        let width = text.len() as u32 * 6;
        let x = frame.width.saturating_sub(width + 2);
        let y = frame.height.saturating_sub(10);
        TextDraw::text(frame, x, y, &text, lum)
    }

    /// True if any pixel in the stamp region is non-zero.
    pub fn is_stamped(frame: &VideoFrame, x: u32, y: u32, width: u32) -> bool {
        let plane = &frame.planes[0];
        let stride = frame.width as usize;
        for row in 0..8 {
            let ry = (y + row) as usize;
            if ry >= frame.height as usize {
                break;
            }
            for col in 0..width as usize {
                let rx = x as usize + col;
                if rx >= stride {
                    break;
                }
                if plane[ry * stride + rx] != 0 {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn format() {
        assert_eq!(TimestampStamp::format(0), "00:00:00");
        assert_eq!(TimestampStamp::format(3661), "01:01:01");
        assert_eq!(
            TimestampStamp::format_full(2026, 10, 17, 60),
            "2026-10-17 00:01:00"
        );
        let n = TimestampStamp::now();
        assert_eq!(n.len(), 8);
        assert!(n.contains(':'));
        assert_eq!(
            TimestampStamp::format_frame(60, 3),
            "00:01:00 f3"
        );
        assert_eq!(TimestampStamp::format_progress(3, 10), "3/10");
        assert_eq!(TimestampStamp::lum("white"), 255);
        assert_eq!(TimestampStamp::lum("black"), 0);
        assert_eq!(TimestampStamp::blend(255, 0.5), 127);
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 64, 16).unwrap();
        let before = f.planes[0][0];
        TimestampStamp::stamp(&mut f, 0, 0, 60, 255).unwrap();
        assert_ne!(f.planes[0][1], before);
        let mut f2 = VideoFrame::new(VideoFormat::Y800, 64, 16).unwrap();
        TimestampStamp::stamp_corner(&mut f2, 60, 255).unwrap();
        assert!(f2.planes[0].iter().any(|&p| p == 255));
    }
}
