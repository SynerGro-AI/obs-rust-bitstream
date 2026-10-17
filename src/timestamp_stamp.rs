//! Safe pure TimestampStamp (burn time into a frame).
//!
//! Formats seconds as HH:MM:SS and draws it onto
//! a frame. No unwraps.
//!
//! Differential: a stamp changes pixels.

use crate::draw::DrawError;
use crate::text_draw::TextDraw;
use crate::video_frame::VideoFrame;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn format() {
        assert_eq!(TimestampStamp::format(0), "00:00:00");
        assert_eq!(TimestampStamp::format(3661), "01:01:01");
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
