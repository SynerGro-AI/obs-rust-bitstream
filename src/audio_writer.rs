//! Safe pure AudioWriter (raw samples core).
//!
//! Writes f32 samples as little-endian bytes.
//! No unwraps.
//!
//! Differential: write + read round-trips.

use std::io::{Read, Write};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioWriteError {
    Io,
}

pub struct AudioWriter;

impl AudioWriter {
    /// Write samples as f32 little-endian.
    pub fn write<W: Write>(samples: &[f32], mut out: W) -> Result<(), AudioWriteError> {
        for s in samples {
            out.write_all(&s.to_le_bytes())
                .map_err(|_| AudioWriteError::Io)?;
        }
        Ok(())
    }

    /// Read samples back.
    pub fn read<R: Read>(mut input: R) -> Result<Vec<f32>, AudioWriteError> {
        let mut buf = Vec::new();
        input
            .read_to_end(&mut buf)
            .map_err(|_| AudioWriteError::Io)?;
        if buf.len() % 4 != 0 {
            return Err(AudioWriteError::Io);
        }
        let mut samples = Vec::new();
        for chunk in buf.chunks(4) {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(chunk);
            samples.push(f32::from_le_bytes(bytes));
        }
        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn differential() {
        let samples = vec![0.0, 0.5, 1.0];
        let mut buf = Vec::new();
        AudioWriter::write(&samples, &mut buf).unwrap();
        let back = AudioWriter::read(Cursor::new(buf)).unwrap();
        assert_eq!(back, samples);
    }
}
