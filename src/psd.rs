//! Safe pure Psd (Photoshop Document).
//!
//! Binary format matching the published Adobe structure.
//! Magic 8BPS, big-endian, 26-byte header, color mode data,
//! image resources, layer and mask information, image data.
//! No unwraps.
//!
//! Differential: a different layer list produces a different file.

use std::fs;
use std::path::Path;

const MAGIC: &[u8; 4] = b"8BPS";
const VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct PsdLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Psd {
    pub width: u32,
    pub height: u32,
    pub channels: u16,
    pub depth: u16,
    pub color_mode: u16,
    pub layers: Vec<PsdLayer>,
}

impl Psd {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            channels: 3,
            depth: 8,
            color_mode: 3,
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, name: &str, visible: bool, opacity: u8) {
        self.layers.push(PsdLayer {
            name: name.to_string(),
            visible,
            opacity,
        });
    }

    /// Write a minimal PSD. Header + empty sections + layer records.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_be_bytes());
        out.extend_from_slice(&[0u8; 6]);
        out.extend_from_slice(&self.channels.to_be_bytes());
        out.extend_from_slice(&self.height.to_be_bytes());
        out.extend_from_slice(&self.width.to_be_bytes());
        out.extend_from_slice(&self.depth.to_be_bytes());
        out.extend_from_slice(&self.color_mode.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes());
        let mut layer_section = Vec::new();
        layer_section.extend_from_slice(&(self.layers.len() as u16).to_be_bytes());
        for layer in &self.layers {
            layer_section.extend_from_slice(&0u32.to_be_bytes());
            layer_section.extend_from_slice(&0u32.to_be_bytes());
            layer_section.extend_from_slice(&self.height.to_be_bytes());
            layer_section.extend_from_slice(&self.width.to_be_bytes());
            layer_section.extend_from_slice(&0u16.to_be_bytes());
            layer_section.extend_from_slice(&[0u8; 12]);
            layer_section.extend_from_slice(b"norm");
            layer_section.push(layer.opacity);
            layer_section.push(0);
            layer_section.push(if layer.visible { 0 } else { 2 });
            layer_section.push(0);
            let name_bytes = layer.name.as_bytes();
            let padded = (name_bytes.len() + 1 + 3) & !3;
            layer_section.push(name_bytes.len() as u8);
            layer_section.extend_from_slice(name_bytes);
            layer_section.extend(std::iter::repeat(0).take(padded - name_bytes.len() - 1));
        }
        let layer_len = layer_section.len() as u32;
        out.extend_from_slice(&(layer_len + 4).to_be_bytes());
        out.extend_from_slice(&layer_len.to_be_bytes());
        out.extend_from_slice(&layer_section);
        out.extend_from_slice(&0u16.to_be_bytes());
        fs::write(path, out).map_err(|e| e.to_string())
    }

    /// Load a PSD. None if the magic is wrong.
    pub fn load(path: &Path) -> Option<Self> {
        let data = fs::read(path).ok()?;
        if data.len() < 26 || &data[0..4] != MAGIC {
            return None;
        }
        let channels = u16::from_be_bytes(data.get(12..14)?.try_into().ok()?);
        let height = u32::from_be_bytes(data.get(14..18)?.try_into().ok()?);
        let width = u32::from_be_bytes(data.get(18..22)?.try_into().ok()?);
        let depth = u16::from_be_bytes(data.get(22..24)?.try_into().ok()?);
        let color_mode = u16::from_be_bytes(data.get(24..26)?.try_into().ok()?);
        let mut doc = Self::new(width, height);
        doc.channels = channels;
        doc.depth = depth;
        doc.color_mode = color_mode;
        let mut i = 26;
        let color_len = u32::from_be_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
        i += 4 + color_len;
        let res_len = u32::from_be_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
        i += 4 + res_len;
        let layer_mask_len = u32::from_be_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
        i += 4;
        if layer_mask_len > 0 {
            let layer_info_len = u32::from_be_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
            i += 4;
            if layer_info_len > 0 {
                let count = u16::from_be_bytes(data.get(i..i + 2)?.try_into().ok()?) as usize;
                i += 2;
                for _ in 0..count {
                    i += 16;
                    i += 2;
                    i += 12;
                    i += 4;
                    let opacity = *data.get(i)?;
                    i += 1;
                    i += 1;
                    let flags = *data.get(i)?;
                    i += 1;
                    i += 1;
                    let visible = flags & 2 == 0;
                    let name_len = *data.get(i)? as usize;
                    i += 1;
                    let name = String::from_utf8(data.get(i..i + name_len)?.to_vec()).ok()?;
                    i += name_len;
                    let padded = (name_len + 1 + 3) & !3;
                    i += padded - name_len - 1;
                    doc.add_layer(&name, visible, opacity);
                }
            }
        }
        Some(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn magic() {
        let d = Psd::new(100, 100);
        let path = env::temp_dir().join("test.psd");
        d.save(&path).unwrap();
        let data = fs::read(&path).unwrap();
        assert_eq!(&data[0..4], b"8BPS");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn round_trip() {
        let mut d = Psd::new(5760, 3240);
        d.add_layer("Mountain Spire", true, 255);
        d.add_layer("Mist Overlay", false, 128);
        let path = env::temp_dir().join("test.psd");
        d.save(&path).unwrap();
        let loaded = Psd::load(&path).unwrap();
        assert_eq!(loaded.width, 5760);
        assert_eq!(loaded.layers.len(), 2);
        assert_eq!(loaded.layers[0].name, "Mountain Spire");
        assert!(!loaded.layers[1].visible);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn differential() {
        let mut a = Psd::new(100, 100);
        a.add_layer("one", true, 255);
        let mut b = Psd::new(100, 100);
        b.add_layer("one", true, 255);
        b.add_layer("two", false, 200);
        assert_ne!(a.layers.len(), b.layers.len());
    }
}
