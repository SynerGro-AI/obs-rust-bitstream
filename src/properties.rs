//! Safe pure Properties (selected layer panel).
//!
//! Shows the selected layer's name, opacity, blend,
//! and visibility. set_opacity and set_visible change
//! the layer. Missing layers return None. No unwraps.
//!
//! Differential: a higher opacity reads as a higher value.

use crate::layer_stack::{BlendMode, LayerStack};

#[derive(Clone, Debug, PartialEq)]
pub struct LayerProps {
    pub name: String,
    pub opacity: f32,
    pub blend: BlendMode,
    pub visible: bool,
}

pub struct Properties;

impl Properties {
    /// Properties of the named layer. None if missing.
    pub fn of(stack: &LayerStack, name: &str) -> Option<LayerProps> {
        stack
            .layers()
            .iter()
            .find(|l| l.name() == name)
            .map(|l| LayerProps {
                name: l.name().to_string(),
                opacity: l.opacity(),
                blend: l.blend(),
                visible: l.visible(),
            })
    }

    /// Set opacity. Returns false if the layer is missing.
    pub fn set_opacity(stack: &mut LayerStack, name: &str, opacity: f32) -> bool {
        stack.set_opacity(name, opacity).is_ok()
    }

    /// Set visibility. Returns false if the layer is missing.
    pub fn set_visible(stack: &mut LayerStack, name: &str, visible: bool) -> bool {
        stack.set_visible(name, visible).is_ok()
    }

    /// One line: "name opacity% blend eye"
    pub fn line(props: &LayerProps) -> String {
        let eye = if props.visible { "●" } else { "○" };
        format!(
            "{} {}% {:?} {}",
            props.name,
            (props.opacity * 100.0).round() as u32,
            props.blend,
            eye
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_layer() {
        let mut s = LayerStack::new();
        s.add("Layer 1").unwrap();
        let p = Properties::of(&s, "Layer 1").unwrap();
        assert_eq!(p.name, "Layer 1");
        assert!(p.visible);
        assert!(Properties::line(&p).contains("Layer 1"));
    }

    #[test]
    fn set_changes() {
        let mut s = LayerStack::new();
        s.add("Layer 1").unwrap();
        assert!(Properties::set_opacity(&mut s, "Layer 1", 0.5));
        assert!(Properties::set_visible(&mut s, "Layer 1", false));
        let p = Properties::of(&s, "Layer 1").unwrap();
        assert!((p.opacity - 0.5).abs() < 0.01);
        assert!(!p.visible);
    }

    #[test]
    fn missing_is_none() {
        let s = LayerStack::new();
        assert!(Properties::of(&s, "Nope").is_none());
        assert!(!Properties::set_opacity(
            &mut LayerStack::new(),
            "Nope",
            1.0
        ));
    }

    #[test]
    fn differential() {
        let mut s = LayerStack::new();
        s.add("A").unwrap();
        Properties::set_opacity(&mut s, "A", 0.2);
        let low = Properties::of(&s, "A").unwrap().opacity;
        Properties::set_opacity(&mut s, "A", 0.9);
        let high = Properties::of(&s, "A").unwrap().opacity;
        assert!(high > low);
    }
}
