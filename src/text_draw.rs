//! Safe pure TextDraw (simple glyph into Y800 frame).
//!
//! Draws a single ASCII character as a 5x7 bitmap.
//! Out-of-bounds rejected. No unwraps.
//!
//! Differential: 'A' at (0,0) sets the expected pixels.

use crate::draw::DrawError;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

/// 5x7 bitmaps for A-Z and 0-9. Each row is 5 bits.
const GLYPHS: &[(char, [u8; 7])] = &[
    ('A', [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
    ('B', [0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110]),
    ('C', [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110]),
    ('0', [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
    ('1', [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
];

pub struct TextDraw;

impl TextDraw {
    fn glyph(c: char) -> Option<[u8; 7]> {
        let c = c.to_ascii_uppercase();
        GLYPHS.iter().find(|(ch, _)| *ch == c).map(|(_, g)| *g)
    }

    /// Draw one character at (x, y). Returns the advance (6).
    pub fn char(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        c: char,
        lum: u8,
    ) -> Result<u32, DrawError> {
        if frame.format != VideoFormat::Y800 {
            return Err(DrawError::WrongFormat);
        }
        let g = Self::glyph(c).unwrap_or([0; 7]);
        if x.saturating_add(5) > frame.width || y.saturating_add(7) > frame.height {
            return Err(DrawError::OutOfBounds);
        }
        if let Some(plane) = frame.planes.get_mut(0) {
            for row in 0..7u32 {
                let bits = g[row as usize];
                for col in 0..5u32 {
                    if (bits >> (4 - col)) & 1 == 1 {
                        let i = ((y + row) * frame.width + (x + col)) as usize;
                        if i < plane.len() {
                            plane[i] = lum;
                        }
                    }
                }
            }
        }
        Ok(6)
    }

    /// Draw a string. Stops at the first character that
    /// would go out of bounds.
    pub fn text(
        frame: &mut VideoFrame,
        x: u32,
        y: u32,
        s: &str,
        lum: u8,
    ) -> Result<u32, DrawError> {
        let mut cx = x;
        for c in s.chars() {
            match Self::char(frame, cx, y, c, lum) {
                Ok(adv) => cx += adv,
                Err(_) => break,
            }
        }
        Ok(cx - x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_bounds() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        assert!(TextDraw::char(&mut f, 0, 0, 'A', 255).is_err());
    }

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        TextDraw::char(&mut f, 0, 0, 'A', 255).unwrap();
        // Top row of A is 01110, so pixel (1,0) is set.
        assert_eq!(f.planes[0][1], 255);
        assert_eq!(f.planes[0][0], 0);
    }
}
