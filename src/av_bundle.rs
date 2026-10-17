//! Safe pure AvBundle (video + audio pair core).
//!
//! Holds H.264 bytes and audio samples together.
//! No unwraps.
//!
//! Differential: push adds both.

#[derive(Clone, Debug, Default)]
pub struct AvBundle {
    video: Vec<u8>,
    audio: Vec<f32>,
}

impl AvBundle {
    pub fn new() -> Self {
        Self::default()
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
        assert_eq!(b.audio()[0], 0.5);
    }
}
