//! Safe pure Preview (session frame viewer core).
//!
//! Shows the current session frame: video size and audio
//! level. No unwraps.
//!
//! Differential: a session with a clip and a track previews.

use crate::pixel_canvas::PixelCanvas;
use crate::session::Session;

#[derive(Clone, Debug)]
pub struct PreviewFrame {
    pub width: u32,
    pub height: u32,
    pub audio_level: f32,
}

pub struct Preview;

impl Preview {
    /// Show the session at `time`.
    pub fn show(session: &Session, time: f32, canvas: &mut PixelCanvas) -> PreviewFrame {
        let f = session.emit(time, canvas);
        PreviewFrame {
            width: f.video.width,
            height: f.video.height,
            audio_level: f.audio.level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show() {
        let mut s = Session::new();
        s.video_mut().add("intro", 0.0, 2.0, "cam1").unwrap();
        s.audio_mut().add("voice", 0.0).unwrap();
        let mut c = PixelCanvas::new(8, 8).unwrap();
        let p = Preview::show(&s, 1.0, &mut c);
        assert_eq!(p.width, 8);
        assert!(p.audio_level > 0.0);
    }

    #[test]
    fn differential() {
        let s = Session::new();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        let p = Preview::show(&s, 0.0, &mut c);
        assert_eq!(p.audio_level, 0.0);
    }
}
