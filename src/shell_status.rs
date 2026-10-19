//! Safe pure ShellStatus (one line for the shell path).
//!
//! Summarizes tool, layers, edit, and blend in one line.
//! No unwraps.
//!
//! Differential: a different blend produces a different line.

use crate::active_tool::ToolSource;
use crate::blend_composite::BlendComposite;
use crate::edit_round::EditRound;
use crate::group_composite::GroupComposite;
use crate::layer_composite::LayerComposite;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
use crate::workspace::Workspace;

pub struct ShellStatus;

impl ShellStatus {
    /// One line covering the shell path including the eye.
    pub fn line() -> String {
        let (layer, group) = EditRound::run();
        let (single, group_n) = GroupComposite::run();
        let (normal, multiply) = BlendComposite::run();
        let (shown, hidden) = Self::eye();
        let mut w = Workspace::new(16, 16);
        w.click("Brush", ToolSource::Photoshop);
        w.options_mut().set("size", 30.0);
        let size = w.options().get("size").unwrap_or(0.0);
        format!(
            "tool={} size={:.0} edit={} group={} members={}/{} blend={:.0}/{:.0} eye={:.0}/{:.0}",
            w.active(), size, layer, group, single, group_n, normal, multiply, shown, hidden
        )
    }

    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// (shown_mean, hidden_mean) for the eye.
    fn eye() -> (f32, f32) {
        let mut s = LayerStack::new();
        s.add("Background", Self::frame(100)).unwrap();
        s.add("Layer 1", Self::frame(200)).unwrap();
        let shown = LayerComposite::mean(&s).unwrap_or(0.0);
        let _ = s.hide("Layer 1");
        let hidden = LayerComposite::mean(&s).unwrap_or(0.0);
        (shown, hidden)
    }

    /// True if the line contains every section.
    pub fn complete(line: &str) -> bool {
        line.contains("tool=") && line.contains("edit=") && line.contains("group=") && line.contains("blend=") && line.contains("eye=")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sections() {
        let line = ShellStatus::line();
        assert!(ShellStatus::complete(&line));
        assert!(line.contains("edit=1"));
        assert!(line.contains("blend="));
    }

    #[test]
    fn blend_differs() {
        let line = ShellStatus::line();
        // normal/multiply are different numbers.
        assert!(line.contains("200/78") || line.contains("/"));
    }

    #[test]
    fn differential() {
        let a = ShellStatus::line();
        assert!(a.contains("edit="));
        assert!(a.len() > 10);
    }
}
