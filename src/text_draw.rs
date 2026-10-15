//! Safe pure TextDraw (simple glyph into Y800 frame).
//!
//! Draws a single ASCII character as a 5x7 bitmap.
//! Out-of-bounds rejected. No unwraps.
//!
//! Differential: 'A' at (0,0) sets the expected pixels.

use crate::draw::DrawError;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

/// 5x7 bitmaps. Each row is 5 bits. Space is empty.
fn glyph(c: char) -> [u8; 7] {
    let c = c.to_ascii_uppercase();
    match c {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b11100, 0b10010, 0b10001, 0b10001, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b01110, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 0b00100, 0b00100],
        ',' => [0, 0, 0, 0, 0b00100, 0b00100, 0b01000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100],
        '?' => [0b01110, 0b10001, 0b00010, 0b00100, 0b00100, 0, 0b00100],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}

pub struct TextDraw;

impl TextDraw {

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
        let g = glyph(c);
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
        assert_eq!(f.planes[0][1], 255);
        assert_eq!(f.planes[0][0], 0);
    }

    #[test]
    fn full_set() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 64, 16).unwrap();
        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.!?";
        let adv = TextDraw::text(&mut f, 0, 0, s, 255).unwrap();
        assert!(adv > 0);
    }
}
