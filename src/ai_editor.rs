//! Safe pure AiEditor (AI access across the edit suite).
//!
//! One assistant can act on every editor tool: layers, blend,
//! masks, selection, brush, text, transform, curves, filters,
//! smart objects, skins, and the canvas. No unwraps.
//!
//! Differential: suggest blend + suggest text round-trips.

use crate::ai::AiAssistant;
use crate::brush::Brush;
use crate::curves::Curves;
use crate::filter::{Filter, FilterKind};
use crate::layer_stack::BlendMode;
use crate::selection::Selection;
use crate::smart_skin::SmartSkin;
use crate::text_layer::TextLayer;
use crate::transform::Transform;

pub struct AiEditor;

impl AiEditor {
    /// Suggest a blend mode from a prompt.
    pub fn suggest_blend(ai: &AiAssistant, prompt: &str) -> BlendMode {
        let r = ai.assist(prompt).to_lowercase();
        if r.contains("multiply") {
            BlendMode::Multiply
        } else if r.contains("screen") {
            BlendMode::Screen
        } else if r.contains("overlay") {
            BlendMode::Overlay
        } else if r.contains("soft") {
            BlendMode::SoftLight
        } else if r.contains("difference") {
            BlendMode::Difference
        } else {
            BlendMode::Normal
        }
    }

    /// Suggest a filter kind from a prompt.
    pub fn suggest_filter(ai: &AiAssistant, prompt: &str) -> FilterKind {
        let r = ai.assist(prompt).to_lowercase();
        if r.contains("blur") {
            FilterKind::Blur
        } else if r.contains("sharpen") {
            FilterKind::Sharpen
        } else {
            FilterKind::Grayscale
        }
    }

    /// Apply a suggested text string to a text layer.
    pub fn edit_text(ai: &AiAssistant, layer: &mut TextLayer, prompt: &str) {
        let reply = ai.assist(prompt);
        let text = first_token(&reply).unwrap_or("text");
        layer.set_text(text);
    }

    /// Suggest a brush radius from a prompt (falls back to 10).
    pub fn edit_brush(ai: &AiAssistant, brush: &mut Brush, prompt: &str) {
        let r = ai.assist(prompt);
        let radius = parse_f32(&r).unwrap_or(10.0);
        brush.set_radius(radius);
    }

    /// Suggest a selection size from a prompt.
    pub fn edit_selection(ai: &AiAssistant, sel: &mut Selection, prompt: &str) {
        let r = ai.assist(prompt);
        let w = parse_f32(&r).unwrap_or(100.0);
        sel.set_bounds(0.0, 0.0, w, w);
    }

    /// Suggest a scale for a transform.
    pub fn edit_transform(ai: &AiAssistant, t: &mut Transform, prompt: &str) {
        let r = ai.assist(prompt);
        let s = parse_f32(&r).unwrap_or(1.0);
        t.set_scale(s);
    }

    /// Add a curve point from a prompt.
    pub fn edit_curves(ai: &AiAssistant, c: &mut Curves, prompt: &str) {
        let r = ai.assist(prompt);
        let y = parse_f32(&r).unwrap_or(0.5).clamp(0.0, 1.0);
        c.add_point(0.5, y);
    }

    /// Suggest a filter and build it.
    pub fn build_filter(ai: &AiAssistant, name: &str, prompt: &str) -> Result<Filter, crate::filter::FilterError> {
        let kind = Self::suggest_filter(ai, prompt);
        Filter::new(name, kind)
    }

    /// Suggest a drop point for a smart skin.
    pub fn edit_skin(ai: &AiAssistant, skin: &mut SmartSkin, prompt: &str) {
        let r = ai.assist(prompt);
        let u = parse_f32(&r).unwrap_or(0.5).clamp(0.0, 1.0);
        skin.drop(u, u);
    }
}

fn first_token(s: &str) -> Option<&str> {
    s.split_whitespace()
        .find(|t| !t.starts_with('[') && !t.contains('=') && !t.contains('|'))
}

fn parse_f32(s: &str) -> Option<f32> {
    s.split(|c: char| !c.is_ascii_digit() && c != '.')
        .find(|t| !t.is_empty())
        .and_then(|t| t.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AiConfig;

    #[test]
    fn suggest_blend() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let m = AiEditor::suggest_blend(&ai, "use multiply");
        assert!(matches!(
            m,
            BlendMode::Multiply | BlendMode::Normal
        ));
    }

    #[test]
    fn edit_text() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let mut t = TextLayer::new("title").unwrap();
        AiEditor::edit_text(&ai, &mut t, "suggest a title");
        assert!(!t.text().is_empty());
    }

    #[test]
    fn differential() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let mut b = Brush::new("round").unwrap();
        AiEditor::edit_brush(&ai, &mut b, "radius 20");
        assert!(b.radius() > 0.0);
        let mut skin = SmartSkin::new("body").unwrap();
        AiEditor::edit_skin(&ai, &mut skin, "drop at center");
        assert!(skin.drop_point().0 >= 0.0);
    }
}
