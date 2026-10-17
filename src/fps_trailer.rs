//! Safe pure FpsTrailer (fps after frame data core).
//!
//! Writes and reads an fps trailer. The existing
//! reader ignores it. No unwraps.
//!
//! Differential: write + read round-trips.

use crate::raw_writer::WriteError;
use std::io::{Read, Write};

const TRAILER: &[u8; 4] = b"FPS!";

pub struct FpsTrailer;

impl FpsTrailer {
    /// Append an fps trailer.
    pub fn write<W: Write>(fps: f32, mut out: W) -> Result<(), WriteError> {
        out.write_all(TRAILER).map_err(|_| WriteError::Io)?;
        out.write_all(&fps.to_le_bytes()).map_err(|_| WriteError::Io)?;
        Ok(())
    }

    /// Read an fps trailer from the end of a buffer.
    pub fn read(buf: &[u8]) -> Result<f32, WriteError> {
        if buf.len() < 8 {
            return Err(WriteError::Io);
        }
        let start = buf.len() - 8;
        if &buf[start..start + 4] != TRAILER {
            return Err(WriteError::Io);
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&buf[start + 4..start + 8]);
        Ok(f32::from_le_bytes(bytes))
    }

    /// Read from any Read by consuming all bytes.
    pub fn read_all<R: Read>(mut input: R) -> Result<f32, WriteError> {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf).map_err(|_| WriteError::Io)?;
        Self::read(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn differential() {
        let mut buf = Vec::new();
        FpsTrailer::write(30.0, &mut buf).unwrap();
        let fps = FpsTrailer::read(&buf).unwrap();
        assert_eq!(fps, 30.0);
        let back = FpsTrailer::read_all(Cursor::new(buf)).unwrap();
        assert_eq!(back, 30.0);
    }
}
