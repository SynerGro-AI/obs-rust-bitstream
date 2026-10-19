//! Safe pure RtmpAudio (OBS audio packet → chunk).
//!
//! Wraps audio samples in a type-8 RTMP chunk.
//! Type 8 is the OBS / RTMP audio message type.
//! Empty samples still produce a valid header.
//! No unwraps.
//!
//! Differential: more samples produce more chunk bytes.

use crate::rtmp_chunk::{RtmpChunk, RTMP_TYPE_AUDIO};

pub struct RtmpAudio;

impl RtmpAudio {
    /// Wrap samples in a type-8 chunk on channel 4.
    pub fn chunk(timestamp: u32, samples: &[u8]) -> Vec<u8> {
        RtmpChunk::wrap(4, RTMP_TYPE_AUDIO, timestamp, samples)
    }

    /// True if the chunk is an audio message.
    pub fn is_audio(chunk: &[u8]) -> bool {
        chunk.len() >= 12 && chunk[7] == RTMP_TYPE_AUDIO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let c = RtmpAudio::chunk(0, &[1, 2, 3, 4]);
        assert!(RtmpAudio::is_audio(&c));
        assert_eq!(c.len(), 12 + 4);
    }

    #[test]
    fn empty_is_header() {
        let c = RtmpAudio::chunk(0, &[]);
        assert!(RtmpAudio::is_audio(&c));
        assert_eq!(c.len(), 12);
    }

    #[test]
    fn differential() {
        let a = RtmpAudio::chunk(0, &[1]);
        let b = RtmpAudio::chunk(0, &[1, 2, 3, 4]);
        assert!(b.len() > a.len());
    }
}
