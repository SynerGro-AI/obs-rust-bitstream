//! Safe pure FileWriter (export to bytes core).
//!
//! Turns an export record into a byte payload. No unwraps.
//!
//! Differential: a gif export produces a non-empty payload
//! that starts with the format name.

use crate::export::Export;
use crate::replay_save::SaveFormat;

pub struct FileWriter;

impl FileWriter {
    /// Encode an export as bytes: format name, path, frame count.
    pub fn encode(export: &Export) -> Vec<u8> {
        let tag = match export.format() {
            SaveFormat::Clip => "CLIP",
            SaveFormat::Gif => "GIF",
            SaveFormat::Video => "VIDEO",
        };
        format!("{tag}|{}|{}", export.path(), export.frames()).into_bytes()
    }

    pub fn starts_with(payload: &[u8], tag: &str) -> bool {
        payload.starts_with(tag.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instant_replay::InstantReplay;
    use crate::replay_save::ReplaySave;

    #[test]
    fn encode() {
        let mut s = ReplaySave::new();
        let mut r = InstantReplay::new(3);
        r.push(1);
        r.push(2);
        s.save_as("loop", &r, SaveFormat::Gif).unwrap();
        let e = Export::write(&s, "loop", "loop.gif").unwrap();
        let bytes = FileWriter::encode(&e);
        assert!(FileWriter::starts_with(&bytes, "GIF"));
        assert!(bytes.len() > 3);
    }
}
