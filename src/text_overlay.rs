//! Safe pure TextOverlay (text on top of a frame).
//!
//! Draws text into a Y800 frame without clearing the
//! rest of the frame. Out-of-bounds text is clipped.
//! No unwraps.
//!
//! Differential: overlay sets the glyph pixels and
//! leaves the background pixel alone.

use crate::draw::Draw;
use crate::text_draw::TextDraw;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlayError {
    Empty,
}

pub struct TextOverlay;

impl TextOverlay {
    /// Draw `text` at (x, y) with luminance `lum`.
    /// Returns the number of characters drawn.
    pub fn draw(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        text: &str,
        lum: u8,
    ) -> Result<usize, OverlayError> {
        if text.is_empty() {
            return Err(OverlayError::Empty);
        }
        let mut drawn = 0;
        let mut cx = x;
        for c in text.chars() {
            if TextDraw::char(frame, cx, y, c, lum).is_ok() {
                drawn += 1;
                cx += 6;
            } else {
                break;
            }
        }
        Ok(drawn)
    }

    /// Draw a border around the text, then the text.
    /// Padding is pixels around the glyphs.
    pub fn bordered(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        text: &str,
        text_lum: u8,
        border_lum: u8,
        padding: u32,
    ) -> Result<usize, OverlayError> {
        if text.is_empty() {
            return Err(OverlayError::Empty);
        }
        let w = (text.chars().count() as u32) * 6 + padding * 2;
        let h = 7 + padding * 2;
        let bx = x.saturating_sub(padding);
        let by = y.saturating_sub(padding);
        // Top and bottom
        let _ = Draw::rect(frame, bx, by, w, 1, border_lum);
        let _ = Draw::rect(frame, bx, by + h.saturating_sub(1), w, 1, border_lum);
        // Left and right
        let _ = Draw::rect(frame, bx, by, 1, h, border_lum);
        let _ = Draw::rect(frame, bx + w.saturating_sub(1), by, 1, h, border_lum);
        Self::draw(frame, x, y, text, text_lum)
    }

    /// Draw text with opacity. opacity is 0..=255.
    /// 0 is fully transparent, 255 is fully opaque.
    /// Blends each glyph pixel with the existing value.
    pub fn with_opacity(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        text: &str,
        text_lum: u8,
        opacity: u8,
    ) -> Result<usize, OverlayError> {
        if text.is_empty() {
            return Err(OverlayError::Empty);
        }
        // Draw into a temporary copy, then blend.
        let mut tmp = frame.clone();
        let drawn = Self::draw(&mut tmp, x, y, text, text_lum)?;
        if let (Some(dst), Some(src)) = (frame.planes.get_mut(0), tmp.planes.first()) {
            for (d, s) in dst.iter_mut().zip(src.iter()) {
                if *s != *d {
                    let blended =
                        ((*s as u16 * opacity as u16) + (*d as u16 * (255 - opacity as u16))) / 255;
                    *d = blended as u8;
                }
            }
        }
        Ok(drawn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn empty() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        assert!(TextOverlay::draw(&mut f, 0, 0, "", 255).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        f.planes[0].fill(64);
        TextOverlay::draw(&mut f, 0, 0, "A", 255).unwrap();
        assert_eq!(f.planes[0][1], 255); // glyph pixel
        assert_eq!(f.planes[0][0], 64); // background kept
    }

    #[test]
    fn border() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 32, 16).unwrap();
        TextOverlay::bordered(&mut f, 4, 4, "A", 255, 128, 1).unwrap();
        // Border pixel above the text
        assert_eq!(f.planes[0][3 * 32 + 4], 128);
    }

    #[test]
    fn opacity() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 16, 16).unwrap();
        f.planes[0].fill(0);
        TextOverlay::with_opacity(&mut f, 0, 0, "A", 255, 128).unwrap();
        let v = f.planes[0][1];
        assert!(v > 0 && v < 255);
    }
}
