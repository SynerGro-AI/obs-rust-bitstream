//! Safe pure LayerStack (layer list core).
//!
//! Holds named layers with visibility, opacity, and a
//! blend mode. Empty names rejected. No unwraps.
//!
//! Differential: hide + show round-trips.

use crate::blend::BlendMode;
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,
    pub blend: BlendMode,
    pub frame: VideoFrame,
}

#[derive(Clone, Debug, Default)]
pub struct LayerStack {
    layers: Vec<Layer>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, frame: VideoFrame) -> Result<(), LayerError> {
        if name.is_empty() {
            return Err(LayerError::EmptyName);
        }
        if self.layers.iter().any(|l| l.name == name) {
            return Err(LayerError::Duplicate);
        }
        self.layers.push(Layer {
            name: name.to_string(),
            visible: true,
            opacity: 255,
            blend: BlendMode::Normal,
            frame,
        });
        Ok(())
    }

    pub fn hide(&mut self, name: &str) -> Result<(), LayerError> {
        self.set_visible(name, false)
    }

    pub fn show(&mut self, name: &str) -> Result<(), LayerError> {
        self.set_visible(name, true)
    }

    fn set_visible(&mut self, name: &str, visible: bool) -> Result<(), LayerError> {
        if let Some(l) = self.layers.iter_mut().find(|l| l.name == name) {
            l.visible = visible;
            Ok(())
        } else {
            Err(LayerError::Missing)
        }
    }

    pub fn set_opacity(&mut self, name: &str, opacity: u8) -> Result<(), LayerError> {
        if let Some(l) = self.layers.iter_mut().find(|l| l.name == name) {
            l.opacity = opacity;
            Ok(())
        } else {
            Err(LayerError::Missing)
        }
    }

    pub fn set_blend(&mut self, name: &str, blend: BlendMode) -> Result<(), LayerError> {
        if let Some(l) = self.layers.iter_mut().find(|l| l.name == name) {
            l.blend = blend;
            Ok(())
        } else {
            Err(LayerError::Missing)
        }
    }

    pub fn visible_layers(&self) -> Vec<&Layer> {
        self.layers.iter().filter(|l| l.visible).collect()
    }

    pub fn count(&self) -> usize {
        self.layers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut stack = LayerStack::new();
        let f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        stack.add("bg", f.clone()).unwrap();
        stack.add("fg", f).unwrap();
        stack.hide("fg").unwrap();
        assert_eq!(stack.visible_layers().len(), 1);
        stack.show("fg").unwrap();
        assert_eq!(stack.visible_layers().len(), 2);
    }
}
