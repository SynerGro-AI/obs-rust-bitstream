//! Safe pure Output (frame emit core).
//!
//! Takes a pixel canvas and emits a frame summary: size and
//! a checksum of the first pixel. No unwraps.
//!
//! Differential: a red canvas emits a red checksum.

use crate::pixel_canvas::PixelCanvas;

#[derive(Clone, Debug)]
pub struct OutputFrame {
    pub width: u32,
    pub height: u32,
    pub first: (f32, f32, f32),
}

pub struct Output;

impl Output {
    /// Emit a frame from a canvas.
    pub fn emit(canvas: &PixelCanvas) -> OutputFrame {
        let first = canvas.get(0, 0).unwrap_or((0.0, 0.0, 0.0));
        OutputFrame {
            width: canvas.width(),
            height: canvas.height(),
            first,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit() {
        let mut c = PixelCanvas::new(4, 4).unwrap();
        c.fill(1.0, 0.0, 0.0);
        let f = Output::emit(&c);
        assert_eq!(f.width, 4);
        assert_eq!(f.first, (1.0, 0.0, 0.0));
    }

    #[test]
    fn differential() {
        let c = PixelCanvas::new(2, 2).unwrap();
        let f = Output::emit(&c);
        assert_eq!(f.first, (0.0, 0.0, 0.0));
    }
}
