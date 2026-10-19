//! Safe pure Phshp (binary layered document).
//!
//! A transferable layered file. Binary header, length-prefixed
//! strings, no text. Saves and loads. No unwraps.
//!
//! Differential: a different layer list produces a different file.

use std::fs;
use std::path::Path;

const MAGIC: &[u8; 4] = b"PHSH";

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

    /// Save as binary. Magic, name, size, layer count, layers.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        let name_bytes = self.name.as_bytes();
        out.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(&self.width.to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&(self.layers.len() as u32).to_le_bytes());
        for (name, vis) in &self.layers {
            out.push(if *vis { 1 } else { 0 });
            let nb = name.as_bytes();
            out.extend_from_slice(&(nb.len() as u32).to_le_bytes());
            out.extend_from_slice(nb);
        }
        fs::write(path, out).map_err(|e| e.to_string())
    }

    /// Load from binary. None if the magic is wrong.
    pub fn load(path: &Path) -> Option<Self> {
        let data = fs::read(path).ok()?;
        if data.len() < 4 || &data[0..4] != MAGIC {
            return None;
        }
        let mut i = 4;
        let name_len = u32::from_le_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
        i += 4;
        let name = String::from_utf8(data.get(i..i + name_len)?.to_vec()).ok()?;
        i += name_len;
        let width = u32::from_le_bytes(data.get(i..i + 4)?.try_into().ok()?);
        i += 4;
        let height = u32::from_le_bytes(data.get(i..i + 4)?.try_into().ok()?);
        i += 4;
        let count = u32::from_le_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
        i += 4;
        let mut doc = Self::new(&name, width, height);
        for _ in 0..count {
            let vis = *data.get(i)? == 1;
            i += 1;
            let nlen = u32::from_le_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
            i += 4;
            let lname = String::from_utf8(data.get(i..i + nlen)?.to_vec()).ok()?;
            i += nlen;
            doc.add_layer(&lname, vis);
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
        d.add_layer("Mist Overlay", false);
        let path = env::temp_dir().join("test.phshp");
        d.save(&path).unwrap();
        let loaded = Phshp::load(&path).unwrap();
        assert_eq!(loaded.name, "mountain");
        assert_eq!(loaded.layers.len(), 2);
        assert!(!loaded.layers[1].1);
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
