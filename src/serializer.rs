//! Safe serializer interface (OBS serializer.h core).
//!
//! The C version is a function-pointer struct for read/write/seek.
//! The safe path is a trait with a pure memory implementation.
//! Little-endian helpers match the s_wl* family. No raw pointers,
//! no unwraps, no panics on short writes.
//!
//! Differential: write then read round-trips the same bytes.

#[cfg(test)]
use crate::diff::assert_equal;

pub trait Serializer {
    fn write(&mut self, data: &[u8]) -> usize;
    fn read(&mut self, buf: &mut [u8]) -> usize;
    fn pos(&self) -> usize;
}

/// In-memory serializer backed by a Vec.
#[derive(Clone, Debug, Default)]
pub struct MemSerializer {
    data: Vec<u8>,
    pub pos: usize,
}

impl MemSerializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Serializer for MemSerializer {
    fn write(&mut self, data: &[u8]) -> usize {
        if self.pos >= self.data.len() {
            self.data.extend_from_slice(data);
        } else {
            let end = self.pos + data.len();
            if end > self.data.len() {
                self.data.resize(end, 0);
            }
            self.data[self.pos..end].copy_from_slice(data);
        }
        self.pos += data.len();
        data.len()
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(self.pos);
        let n = available.min(buf.len());
        if n > 0 {
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
        }
        n
    }

    fn pos(&self) -> usize {
        self.pos
    }
}

/// Little-endian helpers matching s_wl*.
pub fn s_w8(s: &mut dyn Serializer, v: u8) {
    s.write(&[v]);
}

pub fn s_wl16(s: &mut dyn Serializer, v: u16) {
    s.write(&v.to_le_bytes());
}

pub fn s_wl32(s: &mut dyn Serializer, v: u32) {
    s.write(&v.to_le_bytes());
}

pub fn s_wl64(s: &mut dyn Serializer, v: u64) {
    s.write(&v.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let mut s = MemSerializer::new();
        s_wl32(&mut s, 0x1234_5678);
        s.pos = 0;
        let mut buf = [0u8; 4];
        assert_eq!(s.read(&mut buf), 4);
        assert_eq!(u32::from_le_bytes(buf), 0x1234_5678);
    }

    #[test]
    fn short_read_safe() {
        let mut s = MemSerializer::new();
        s.write(&[1, 2]);
        s.pos = 0;
        let mut buf = [0u8; 4];
        assert_eq!(s.read(&mut buf), 2);
    }

    #[test]
    fn empty() {
        let mut s = MemSerializer::new();
        let mut buf = [0u8; 1];
        assert_eq!(s.read(&mut buf), 0);
    }

    #[test]
    fn differential() {
        let mut s = MemSerializer::new();
        s_wl16(&mut s, 0xABCD);
        let got = s.as_bytes().to_vec();
        let expected = 0xABCDu16.to_le_bytes().to_vec();
        assert_equal(got, expected, "serializer le16");
    }
}
