//! Safe pure Program (live output core).
//!
//! Emits the session as the live program feed, distinct from
//! the preview. No unwraps.
//!
//! Differential: a session with a clip programs red.

use crate::ai::AiAssistant;
use crate::ai_operator::AiOperator;
use crate::live_camera::LiveCamera;
use crate::output::OutputFrame;
use crate::performance::Performance;
use crate::pixel_canvas::PixelCanvas;
use crate::session::Session;

pub struct Program;

impl Program {
    /// Emit the live program frame from the session.
    pub fn live(session: &Session, time: f32, canvas: &mut PixelCanvas) -> OutputFrame {
        session.emit(time, canvas).video
    }

    /// Emit a live performance: music video behind the singer.
    pub fn live_performance(
        performance: &Performance,
        camera: &LiveCamera,
        canvas: &mut PixelCanvas,
    ) -> OutputFrame {
        let (bg, singer) = performance.frame(camera);
        let color = if performance.is_live() { singer } else { bg };
        canvas.fill(color.0, color.1, color.2);
        crate::output::Output::emit(canvas)
    }

    /// Run the AI crew then emit the live performance.
    pub fn live_with_crew(
        ai: &AiAssistant,
        performance: &Performance,
        camera: &mut LiveCamera,
        topic: &str,
        canvas: &mut PixelCanvas,
    ) -> OutputFrame {
        let _crew = AiOperator::run(ai, camera, topic);
        Self::live_performance(performance, camera, canvas)
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

    #[test]
    fn live_performance() {
        let mut p = Performance::new("set").unwrap();
        p.set_video("mv.mp4");
        p.go_live();
        let mut cam = LiveCamera::new("singer").unwrap();
        cam.go_live();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        let f = Program::live_performance(&p, &cam, &mut c);
        assert_eq!(f.first, (0.2, 0.4, 0.8));
    }

    #[test]
    fn live_with_crew() {
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        let mut p = Performance::new("set").unwrap();
        p.go_live();
        let mut cam = LiveCamera::new("singer").unwrap();
        cam.go_live();
        let mut c = PixelCanvas::new(4, 4).unwrap();
        let f = Program::live_with_crew(&ai, &p, &mut cam, "concert", &mut c);
        assert!(cam.zoom() >= 1.0);
        assert_eq!(f.first, (0.2, 0.4, 0.8));
    }
}
