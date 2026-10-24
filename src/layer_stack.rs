//! Safe pure LayerStack (Photoshop-like layering core).
//!
//! Holds named layers with opacity and a blend mode. Empty
//! and duplicate names rejected. No unwraps.
//!
//! Differential: two layers + composite opacity round-trips.

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Layer {
    name: String,
    opacity: f32,
    blend: BlendMode,
    visible: bool,
}

impl Layer {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    pub fn blend(&self) -> BlendMode {
        self.blend
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
}

#[derive(Clone, Debug, Default)]
pub struct LayerStack {
    layers: Vec<Layer>,
    index: HashMap<String, usize>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str) -> Result<(), LayerError> {
        if name.is_empty() {
            return Err(LayerError::EmptyName);
        }
        if self.index.contains_key(name) {
            return Err(LayerError::Duplicate);
        }
        self.index.insert(name.to_string(), self.layers.len());
        self.layers.push(Layer {
            name: name.to_string(),
            opacity: 1.0,
            blend: BlendMode::Normal,
            visible: true,
        });
        Ok(())
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut Layer, LayerError> {
        let i = *self.index.get(name).ok_or(LayerError::Missing)?;
        Ok(&mut self.layers[i])
    }

    pub fn set_opacity(&mut self, name: &str, opacity: f32) -> Result<(), LayerError> {
        self.get_mut(name)?.opacity = opacity.clamp(0.0, 1.0);
        Ok(())
    }

    pub fn set_blend(&mut self, name: &str, mode: BlendMode) -> Result<(), LayerError> {
        self.get_mut(name)?.blend = mode;
        Ok(())
    }

    pub fn set_visible(&mut self, name: &str, v: bool) -> Result<(), LayerError> {
        self.get_mut(name)?.visible = v;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Remove a layer by name. Returns false if missing.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(i) = self.layers.iter().position(|l| l.name() == name) {
            self.layers.remove(i);
            self.index.remove(name);
            // Rebuild index after the removed position.
            for (n, layer) in self.layers.iter().enumerate() {
                self.index.insert(layer.name().to_string(), n);
            }
            true
        } else {
            false
        }
    }

    /// Composite opacity of all visible layers (product).
    pub fn composite_opacity(&self) -> f32 {
        self.layers
            .iter()
            .filter(|l| l.visible)
            .map(|l| l.opacity)
            .fold(1.0, |acc, o| acc * o)
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two() {
        let mut s = LayerStack::new();
        s.add("bg").unwrap();
        s.add("fg").unwrap();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn duplicate_and_empty() {
        let mut s = LayerStack::new();
        s.add("a").unwrap();
        assert!(matches!(s.add("a"), Err(LayerError::Duplicate)));
        assert!(matches!(s.add(""), Err(LayerError::EmptyName)));
    }

    #[test]
    fn differential() {
        let mut s = LayerStack::new();
        s.add("bg").unwrap();
        s.add("fg").unwrap();
        s.set_opacity("fg", 0.5).unwrap();
        s.set_blend("fg", BlendMode::Multiply).unwrap();
        assert!((s.composite_opacity() - 0.5).abs() < 0.001);
        assert_eq!(s.layers()[1].blend(), BlendMode::Multiply);
    }
}
