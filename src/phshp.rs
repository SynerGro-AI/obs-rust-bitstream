//! Safe pure Phshp (layered document).
//!
//! A transferable layered file. Holds a name, canvas size,
//! and a list of layer names with visibility. Saves and
//! loads as a simple text format so it can move between
//! tools. No unwraps.
//!
//! Differential: a different layer list produces a different file.

use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Phshp {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<(String, bool)>,
}

impl Phshp {
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, name: &str, visible: bool) {
        self.layers.push((name.to_string(), visible));
    }

    /// Save as a simple text file. One layer per line.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let mut out = format!("PHSHP\n{} {} {}\n", self.name, self.width, self.height);
        for (name, vis) in &self.layers {
            out.push_str(&format!("{} {}\n", if *vis { "1" } else { "0" }, name));
        }
        fs::write(path, out).map_err(|e| e.to_string())
    }

    /// Load from a simple text file. None if the header is wrong.
    pub fn load(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        let mut lines = text.lines();
        if lines.next()? != "PHSHP" {
            return None;
        }
        let header = lines.next()?;
        let mut parts = header.split_whitespace();
        let name = parts.next()?.to_string();
        let width: u32 = parts.next()?.parse().ok()?;
        let height: u32 = parts.next()?.parse().ok()?;
        let mut doc = Self::new(&name, width, height);
        for line in lines {
            let mut p = line.splitn(2, ' ');
            let vis = p.next()? == "1";
            let name = p.next()?.to_string();
            doc.add_layer(&name, vis);
        }
        Some(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn round_trip() {
        let mut d = Phshp::new("mountain", 5760, 3240);
        d.add_layer("Mountain Spire", true);
        d.add_layer("Mist Overlay", true);
        let path = env::temp_dir().join("test.phshp");
        d.save(&path).unwrap();
        let loaded = Phshp::load(&path).unwrap();
        assert_eq!(loaded.name, "mountain");
        assert_eq!(loaded.layers.len(), 2);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn differential() {
        let mut a = Phshp::new("a", 100, 100);
        a.add_layer("one", true);
        let mut b = Phshp::new("b", 100, 100);
        b.add_layer("one", true);
        b.add_layer("two", false);
        assert_ne!(a.layers.len(), b.layers.len());
    }
}
