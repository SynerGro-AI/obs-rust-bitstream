//! Safe pure WavReader (PCM WAV core).
//!
//! Reads a minimal PCM WAV (16-bit or 8-bit, mono or
//! stereo) and returns f32 samples. No unwraps.
//!
//! Differential: a known header parses.

use std::io::Read;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WavError {
    BadHeader,
    Unsupported,
    Io,
}

#[derive(Clone, Debug)]
pub struct WavData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

pub struct WavReader;

impl WavReader {
    /// Read a PCM WAV from any Read.
    pub fn read<R: Read>(mut input: R) -> Result<WavData, WavError> {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf).map_err(|_| WavError::Io)?;
        Self::parse(&buf)
    }

    /// Parse a PCM WAV from bytes.
    pub fn parse(data: &[u8]) -> Result<WavData, WavError> {
        if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
            return Err(WavError::BadHeader);
        }
        let mut pos = 12;
        let mut sample_rate = 0u32;
        let mut channels = 0u16;
        let mut bits = 0u16;
        let mut samples = Vec::new();
        while pos + 8 <= data.len() {
            let id = &data[pos..pos + 4];
            let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap_or([0; 4])) as usize;
            pos += 8;
            if pos + size > data.len() {
                break;
            }
            if id == b"fmt " && size >= 16 {
                let format = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap_or([0; 2]));
                if format != 1 {
                    return Err(WavError::Unsupported);
                }
                channels = u16::from_le_bytes(data[pos + 2..pos + 4].try_into().unwrap_or([0; 2]));
                sample_rate = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap_or([0; 4]));
                bits = u16::from_le_bytes(data[pos + 14..pos + 16].try_into().unwrap_or([0; 2]));
            } else if id == b"data" {
                samples = Self::decode(&data[pos..pos + size], channels, bits)?;
            }
            pos += size;
            if size % 2 == 1 {
                pos += 1;
            }
        }
        if samples.is_empty() || sample_rate == 0 {
            return Err(WavError::BadHeader);
        }
        Ok(WavData {
            sample_rate,
            channels,
            samples,
        })
    }

    fn decode(data: &[u8], channels: u16, bits: u16) -> Result<Vec<f32>, WavError> {
        let ch = channels.max(1) as usize;
        match bits {
            8 => {
                let mut out = Vec::new();
                for chunk in data.chunks(ch) {
                    let v = chunk.first().copied().unwrap_or(128);
                    out.push((v as f32 - 128.0) / 128.0);
                }
                Ok(out)
            }
            16 => {
                let mut out = Vec::new();
                let mut i = 0;
                while i + 1 < data.len() {
                    let v = i16::from_le_bytes([data[i], data[i + 1]]);
                    out.push(v as f32 / 32768.0);
                    i += 2 * ch;
                }
                Ok(out)
            }
            _ => Err(WavError::Unsupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad() {
        assert!(WavReader::parse(b"not a wav").is_err());
    }

    #[test]
    fn differential() {
        // minimal 16-bit mono WAV: 1 sample = 0
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&44100u32.to_le_bytes());
        wav.extend_from_slice(&88200u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&2u32.to_le_bytes());
        wav.extend_from_slice(&0i16.to_le_bytes());
        let data = WavReader::parse(&wav).unwrap();
        assert_eq!(data.sample_rate, 44100);
        assert_eq!(data.channels, 1);
        assert_eq!(data.samples.len(), 1);
        assert_eq!(data.samples[0], 0.0);
    }
}
