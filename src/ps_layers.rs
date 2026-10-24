//! Safe pure PsLayers (Photoshop layer panel).
//!
//! Shows the layer stack Photoshop-style: visibility,
//! opacity, blend mode, and name. Top of the list is
//! the top of the stack. No unwraps.
//!
//! Differential: more layers produce more panel lines.

use crate::layer_stack::{BlendMode, Layer, LayerStack};

pub struct PsLayers;

impl PsLayers {
    /// One line per layer, top first: "eye opacity% blend name"
    pub fn panel(stack: &LayerStack) -> Vec<String> {
        stack.layers().iter().rev().map(Self::line).collect()
    }

    fn line(layer: &Layer) -> String {
        let eye = if layer.visible() { "●" } else { "○" };
        let pct = (layer.opacity() * 100.0).round() as u32;
        format!(
            "{} {}% {} {}",
            eye,
            pct,
            Self::blend_name(layer.blend()),
            layer.name()
        )
    }

    fn blend_name(mode: BlendMode) -> &'static str {
        match mode {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
            BlendMode::ColorDodge => "Color Dodge",
            BlendMode::ColorBurn => "Color Burn",
            BlendMode::SoftLight => "Soft Light",
            BlendMode::HardLight => "Hard Light",
            BlendMode::Difference => "Difference",
            BlendMode::Exclusion => "Exclusion",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_top_first() {
        let mut s = LayerStack::new();
        s.add("Background").unwrap();
        s.add("Layer 1").unwrap();
        let panel = PsLayers::panel(&s);
        assert_eq!(panel.len(), 2);
        assert!(panel[0].contains("Layer 1"));
        assert!(panel[0].contains("●"));
        assert!(panel[0].contains("Normal"));
    }

    #[test]
    fn hidden_shows_empty_eye() {
        let mut s = LayerStack::new();
        s.add("Hidden").unwrap();
        s.set_visible("Hidden", false).unwrap();
        let panel = PsLayers::panel(&s);
        assert!(panel[0].starts_with("○"));
    }

    #[test]
    fn differential() {
        let mut a = LayerStack::new();
        let mut b = LayerStack::new();
        a.add("A").unwrap();
        b.add("A").unwrap();
        b.add("B").unwrap();
        assert!(PsLayers::panel(&b).len() > PsLayers::panel(&a).len());
    }
}
