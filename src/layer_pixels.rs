//! Safe pure LayerPixels (per-layer image data).
//!
//! Each layer holds a name, visibility, opacity, and
//! a path to its image. Composite returns the list of
//! visible layers in order. No unwraps.
//!
//! Differential: hiding a layer changes the visible list.

#[derive(Clone, Debug, PartialEq)]
pub struct LayerPixels {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,
    pub path: String,
}

impl LayerPixels {
    pub fn new(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            visible: true,
            opacity: 255,
            path: path.to_string(),
        }
    }
}

pub struct LayerStackPixels {
    pub layers: Vec<LayerPixels>,
}

impl LayerStackPixels {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add(&mut self, layer: LayerPixels) {
        self.layers.push(layer);
    }

    /// Visible layers in order. Empty if none are visible.
    pub fn visible(&self) -> Vec<&LayerPixels> {
        self.layers.iter().filter(|l| l.visible).collect()
    }

    /// Toggle a layer. None if the name is missing.
    pub fn toggle(&mut self, name: &str) -> Option<bool> {
        let layer = self.layers.iter_mut().find(|l| l.name == name)?;
        layer.visible = !layer.visible;
        Some(layer.visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible() {
        let mut s = LayerStackPixels::new();
        s.add(LayerPixels::new("a", "a.jpg"));
        s.add(LayerPixels::new("b", "b.jpg"));
        assert_eq!(s.visible().len(), 2);
    }

    #[test]
    fn toggle() {
        let mut s = LayerStackPixels::new();
        s.add(LayerPixels::new("a", "a.jpg"));
        assert_eq!(s.toggle("a"), Some(false));
        assert_eq!(s.visible().len(), 0);
    }

    #[test]
    fn differential() {
        let mut a = LayerStackPixels::new();
        a.add(LayerPixels::new("a", "a.jpg"));
        let mut b = LayerStackPixels::new();
        b.add(LayerPixels::new("a", "a.jpg"));
        b.toggle("a");
        assert_ne!(a.visible().len(), b.visible().len());
    }
}
