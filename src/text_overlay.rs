//! Safe pure TextOverlay (text on top of a frame).
//!
//! Draws text into a Y800 frame without clearing the
//! rest of the frame. Out-of-bounds text is clipped.
//! No unwraps.
//!
//! Differential: overlay sets the glyph pixels and
//! leaves the background pixel alone.

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
        assert_eq!(f.planes[0][0], 64);  // background kept
    }
}
