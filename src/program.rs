//! Safe pure Program (live output core).
//!
//! Emits the session as the live program feed, distinct from
//! the preview. No unwraps.
//!
//! Differential: a session with a clip programs red.

use crate::output::OutputFrame;
use crate::pixel_canvas::PixelCanvas;
use crate::session::Session;

pub struct Program;

impl Program {
    /// Emit the live program frame from the session.
    pub fn live(session: &Session, time: f32, canvas: &mut PixelCanvas) -> OutputFrame {
        session.emit(time, canvas).video
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live() {
        let mut s = Session::new();
        s.video_mut().add("main", 0.0, 5.0, "cam1").unwrap();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        let f = Program::live(&s, 1.0, &mut c);
        assert_eq!(f.first, (1.0, 0.0, 0.0));
    }

    #[test]
    fn differential() {
        let s = Session::new();
        let mut c = PixelCanvas::new(2, 2).unwrap();
        let f = Program::live(&s, 0.0, &mut c);
        assert_eq!(f.first, (0.0, 0.0, 0.0));
    }
}
