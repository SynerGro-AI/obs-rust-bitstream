//! Safe pure EditorUi (layer / mask / onion panel core).
//!
//! Exposes the layer stack, blend modes, masks, and the onion
//! viewer as a single editor surface. No unwraps.
//!
//! Differential: add layer + set blend + attach mask + onion
//! view round-trips.

use crate::adjustment::{Adjustment, AdjustKind};
use crate::ai::AiAssistant;
use crate::history::History;
use crate::layer_group::GroupSet;
use crate::layer_stack::{BlendMode, LayerStack};
use crate::mask::MaskSet;
use crate::onion_viewer::{OnionFrame, OnionViewer};

#[derive(Clone, Debug)]
pub struct EditorUi {
    layers: LayerStack,
    masks: MaskSet,
    onion: OnionViewer,
    groups: GroupSet,
    adjustments: Vec<Adjustment>,
    history: History,
}

impl EditorUi {
    pub fn new() -> Self {
        Self {
            layers: LayerStack::new(),
            masks: MaskSet::new(),
            onion: OnionViewer::new(),
            groups: GroupSet::new(),
            adjustments: Vec::new(),
            history: History::new(),
        }
    }

    pub fn layers(&self) -> &LayerStack {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut LayerStack {
        &mut self.layers
    }

    pub fn masks(&self) -> &MaskSet {
        &self.masks
    }

    pub fn masks_mut(&mut self) -> &mut MaskSet {
        &mut self.masks
    }

    pub fn onion(&self) -> &OnionViewer {
        &self.onion
    }

    pub fn onion_mut(&mut self) -> &mut OnionViewer {
        &mut self.onion
    }

    /// Set a layer's blend mode.
    pub fn set_blend(&mut self, layer: &str, mode: BlendMode) -> Result<(), crate::layer_stack::LayerError> {
        self.layers.set_blend(layer, mode)
    }

    /// Attach a mask to a layer.
    pub fn attach_mask(&mut self, layer: &str, mask: &str) -> Result<(), crate::mask::MaskError> {
        self.masks.attach(layer, mask)
    }

    /// Onion view for the current frame.
    pub fn onion_view(&self, current: i32, count: i32) -> Vec<OnionFrame> {
        self.onion.view(current, count)
    }

    pub fn groups(&self) -> &GroupSet {
        &self.groups
    }

    pub fn groups_mut(&mut self) -> &mut GroupSet {
        &mut self.groups
    }

    /// Add an adjustment layer.
    pub fn add_adjustment(&mut self, name: &str, kind: AdjustKind) -> Result<(), crate::adjustment::AdjustError> {
        let a = Adjustment::new(name, kind)?;
        self.adjustments.push(a);
        Ok(())
    }

    pub fn adjustment_count(&self) -> usize {
        self.adjustments.len()
    }

    /// Ask the assistant for a blend mode and apply it.
    /// Falls back to Normal if the reply is not a known mode.
    pub fn set_blend_ai(
        &mut self,
        ai: &AiAssistant,
        layer: &str,
        prompt: &str,
    ) -> Result<(), crate::layer_stack::LayerError> {
        let reply = ai.assist(prompt).to_lowercase();
        let mode = if reply.contains("multiply") {
            BlendMode::Multiply
        } else if reply.contains("screen") {
            BlendMode::Screen
        } else if reply.contains("overlay") {
            BlendMode::Overlay
        } else if reply.contains("darken") {
            BlendMode::Darken
        } else if reply.contains("lighten") {
            BlendMode::Lighten
        } else if reply.contains("dodge") {
            BlendMode::ColorDodge
        } else if reply.contains("burn") {
            BlendMode::ColorBurn
        } else if reply.contains("soft") {
            BlendMode::SoftLight
        } else if reply.contains("hard") {
            BlendMode::HardLight
        } else if reply.contains("difference") {
            BlendMode::Difference
        } else if reply.contains("exclusion") {
            BlendMode::Exclusion
        } else {
            BlendMode::Normal
        };
        self.layers.set_blend(layer, mode)
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }

    /// Record an edit step.
    pub fn record(&mut self, step: &str) {
        self.history.push(step);
    }

    pub fn add_adjustment_ai(
        &mut self,
        ai: &AiAssistant,
        name: &str,
        prompt: &str,
    ) -> Result<(), crate::adjustment::AdjustError> {
        let reply = ai.assist(prompt).to_lowercase();
        let kind = if reply.contains("contrast") {
            AdjustKind::Contrast
        } else if reply.contains("hue") {
            AdjustKind::Hue
        } else {
            AdjustKind::Brightness
        };
        self.add_adjustment(name, kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_panel() {
        let mut ui = EditorUi::new();
        ui.layers_mut().add("bg").unwrap();
        ui.layers_mut().add("fg").unwrap();
        ui.set_blend("fg", BlendMode::Multiply).unwrap();
        ui.masks_mut().add("soft").unwrap();
        ui.attach_mask("fg", "soft").unwrap();
        let frames = ui.onion_view(1, 3);
        assert_eq!(ui.layers().len(), 2);
        assert!(ui.masks().mask_of("fg").is_some());
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn differential() {
        let mut ui = EditorUi::new();
        ui.layers_mut().add("fg").unwrap();
        ui.set_blend("fg", BlendMode::Screen).unwrap();
        assert_eq!(ui.layers().layers()[0].blend(), BlendMode::Screen);
    }

    #[test]
    fn groups_and_adjustments() {
        let mut ui = EditorUi::new();
        ui.layers_mut().add("fg").unwrap();
        ui.groups_mut().add("foreground").unwrap();
        ui.groups_mut().add_member("foreground", "fg").unwrap();
        ui.add_adjustment("bright", AdjustKind::Brightness).unwrap();
        assert_eq!(ui.groups().get("foreground").unwrap().members().len(), 1);
        assert_eq!(ui.adjustment_count(), 1);
    }

    #[test]
    fn ai_blend_and_adjust() {
        let mut ui = EditorUi::new();
        ui.layers_mut().add("fg").unwrap();
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        ui.set_blend_ai(&ai, "fg", "suggest a blend").unwrap();
        ui.add_adjustment_ai(&ai, "auto", "suggest an adjustment")
            .unwrap();
        assert_eq!(ui.adjustment_count(), 1);
    }
}
