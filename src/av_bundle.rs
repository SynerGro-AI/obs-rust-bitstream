//! Safe pure AvBundle (video + audio pair core).
//!
//! Holds H.264 bytes and audio samples together.
//! Can write and read a simple file. No unwraps.
//!
//! Differential: write + read round-trips.

use std::io::{Read, Write};

use crate::encoder_select::EncoderChoice;

const MAGIC: &[u8; 4] = b"AVBN";

#[derive(Clone, Debug)]
pub struct AvBundle {
    video: Vec<u8>,
    audio: Vec<f32>,
    fps: f32,
    encoder: EncoderChoice,
}

impl AvBundle {
    pub fn new() -> Self {
        Self {
            video: Vec::new(),
            audio: Vec::new(),
            fps: 0.0,
            encoder: EncoderChoice::Software,
        }
    }

    pub fn push_video(&mut self, bytes: &[u8]) {
        self.video.extend_from_slice(bytes);
    }

    pub fn push_audio(&mut self, samples: &[f32]) {
        self.audio.extend_from_slice(samples);
    }

    pub fn video(&self) -> &[u8] {
        &self.video
    }

    pub fn audio(&self) -> &[f32] {
        &self.audio
    }

    pub fn video_len(&self) -> usize {
        self.video.len()
    }

    pub fn audio_len(&self) -> usize {
        self.audio.len()
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn set_fps(&mut self, fps: f32) {
        if fps > 0.0 {
            self.fps = fps;
        }
    }

    pub fn encoder(&self) -> EncoderChoice {
        self.encoder
    }

    pub fn set_encoder(&mut self, choice: EncoderChoice) {
        self.encoder = choice;
    }

    /// Duration in milliseconds from audio length and fps.
    pub fn duration_ms(&self) -> f32 {
        if self.fps <= 0.0 {
            return 0.0;
        }
        self.audio.len() as f32 / self.fps * 1000.0
    }

    /// Write the bundle: magic + video_len + audio_len + fps + video + audio.
    pub fn write<W: Write>(&self, mut out: W) -> Result<(), std::io::Error> {
        out.write_all(MAGIC)?;
        out.write_all(&(self.video.len() as u32).to_le_bytes())?;
        out.write_all(&(self.audio.len() as u32).to_le_bytes())?;
        out.write_all(&self.fps.to_le_bytes())?;
        out.write_all(&[self.encoder as u8])?;
        out.write_all(&self.video)?;
        for s in &self.audio {
            out.write_all(&s.to_le_bytes())?;
        }
        Ok(())
    }

    /// Read a bundle back.
    pub fn read<R: Read>(mut input: R) -> Result<Self, std::io::Error> {
        let mut magic = [0u8; 4];
        input.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad magic",
            ));
        }
        let mut buf = [0u8; 4];
        input.read_exact(&mut buf)?;
        let video_len = u32::from_le_bytes(buf) as usize;
        input.read_exact(&mut buf)?;
        let audio_len = u32::from_le_bytes(buf) as usize;
        input.read_exact(&mut buf)?;
        let fps = f32::from_le_bytes(buf);
        let mut enc_byte = [0u8; 1];
        input.read_exact(&mut enc_byte)?;
        let encoder = match enc_byte[0] {
            1 => EncoderChoice::Nvidia,
            2 => EncoderChoice::Amd,
            3 => EncoderChoice::Intel,
            _ => EncoderChoice::Software,
        };
        let mut video = vec![0u8; video_len];
        input.read_exact(&mut video)?;
        let mut audio = Vec::with_capacity(audio_len);
        for _ in 0..audio_len {
            input.read_exact(&mut buf)?;
            audio.push(f32::from_le_bytes(buf));
        }
        Ok(Self {
            video,
            audio,
            fps,
            encoder,
        })
    }
}

impl Default for AvBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut b = AvBundle::new();
        b.push_video(&[0, 0, 0, 1]);
        b.push_audio(&[0.5, 1.0]);
        assert_eq!(b.video_len(), 4);
        assert_eq!(b.audio_len(), 2);
        b.set_fps(30.0);
        b.set_encoder(EncoderChoice::Software);
        let mut buf = Vec::new();
        b.write(&mut buf).unwrap();
        let back = AvBundle::read(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(back.video_len(), 4);
        assert_eq!(back.audio_len(), 2);
        assert_eq!(back.fps(), 30.0);
        assert_eq!(back.encoder(), EncoderChoice::Software);
        assert_eq!(back.audio()[0], 0.5);
    }
}
