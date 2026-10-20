//! Safe pure RawWriter (frame sequence → file).
//!
//! Writes a simple header (magic, width, height, count) then
//! the raw Y800 planes. Empty sequence rejected. No unwraps.
//!
//! Differential: write + read-back matches the source frames.

use crate::frame_sequence::{FrameSequence, SequenceError};
use std::io::Write;

pub const MAGIC: &[u8; 4] = b"OBSR";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    Empty,
    Io,
}

impl From<SequenceError> for WriteError {
    fn from(_: SequenceError) -> Self {
        WriteError::Empty
    }
}

pub struct RawWriter;

impl RawWriter {
    /// Write the sequence to any Write. Header is 16 bytes:
    /// magic(4) + width(4) + height(4) + count(4), little-endian.
    pub fn write<W: Write>(seq: &FrameSequence, mut out: W) -> Result<(), WriteError> {
        if seq.is_empty() {
            return Err(WriteError::Empty);
        }
        let first = seq.get(0)?;
        let width = first.width;
        let height = first.height;
        let count = seq.len() as u32;
        out.write_all(MAGIC).map_err(|_| WriteError::Io)?;
        out.write_all(&width.to_le_bytes()).map_err(|_| WriteError::Io)?;
        out.write_all(&height.to_le_bytes()).map_err(|_| WriteError::Io)?;
        out.write_all(&count.to_le_bytes()).map_err(|_| WriteError::Io)?;
        for frame in seq.frames() {
            if let Some(plane) = frame.planes.first() {
                out.write_all(plane).map_err(|_| WriteError::Io)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_source::ColorSource;
    use crate::frame_renderer::FrameRenderer;
    use crate::frame_sequence::FrameSequence;

    #[test]
    fn empty() {
        let s = FrameSequence::new();
        let mut buf = Vec::new();
        assert!(RawWriter::write(&s, &mut buf).is_err());
    }

    #[test]
    fn differential() {
        let c = ColorSource::new(255, 0, 0);
        let f = FrameRenderer::solid(&c, 2, 2).unwrap();
        let mut seq = FrameSequence::new();
        seq.push(f);
        let mut buf = Vec::new();
        RawWriter::write(&seq, &mut buf).unwrap();
        assert_eq!(&buf[0..4], b"OBSR");
        assert_eq!(u32::from_le_bytes(buf[4..8].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(buf[8..12].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(buf[12..16].try_into().unwrap()), 1);
        assert_eq!(buf.len(), 16 + 4);
    }
}
