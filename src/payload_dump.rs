//! Safe pure PayloadDump (H.264 packets → length-prefixed file).
//!
//! Pulls N frames from a Capture, encodes them with H264Codec,
//! and writes a 12-byte header (count + total bytes) followed by
//! a PacketFile payload. Reads the packets back. No unwraps.
//!
//! Differential: more frames produce more packet bytes.

use crate::capture::Capture;
use crate::h264_codec::H264Codec;
use crate::packet_file::PacketFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadError {
    Encode,
    NoFrame,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadHeader {
    pub count: u32,
    pub bytes: u64,
}

pub struct PayloadDump;

impl PayloadDump {
    /// Encode `frames` from the capture and write header + PacketFile.
    pub fn write(
        codec: &mut H264Codec,
        capture: &mut dyn Capture,
        frames: usize,
    ) -> Result<Vec<u8>, PayloadError> {
        let mut packets = Vec::new();
        for i in 0..frames {
            let frame = capture.frame().ok_or(PayloadError::NoFrame)?;
            if let Some(pkt) = codec
                .encode_packet(&frame, i as i64)
                .map_err(|_| PayloadError::Encode)?
            {
                packets.push(pkt);
            }
        }
        let flushed = codec.flush();
        if !flushed.is_empty() {
            packets.push(crate::encoder_packet::EncoderPacket::new(
                crate::encoder_context::EncoderType::Video,
                frames as i64,
                frames as i64,
                flushed,
            ));
        }
        let total: u64 = packets.iter().map(|p| p.data.len() as u64).sum();
        let mut buf = Vec::new();
        buf.extend_from_slice(&(packets.len() as u32).to_le_bytes());
        buf.extend_from_slice(&total.to_le_bytes());
        buf.extend_from_slice(&PacketFile::write(&packets));
        Ok(buf)
    }

    /// Read the header. Returns None if shorter than 12 bytes.
    pub fn header(buf: &[u8]) -> Option<PayloadHeader> {
        if buf.len() < 12 {
            return None;
        }
        let count = u32::from_le_bytes(buf[0..4].try_into().ok()?);
        let bytes = u64::from_le_bytes(buf[4..12].try_into().ok()?);
        Some(PayloadHeader { count, bytes })
    }

    /// Read the packet payloads after the header.
    pub fn packets(buf: &[u8]) -> Option<Vec<Vec<u8>>> {
        if buf.len() < 12 {
            return None;
        }
        PacketFile::read(&buf[12..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::SyntheticCapture;
    use crate::live_weights::LiveWeights;

    fn codec() -> H264Codec {
        let mut weights = LiveWeights::new();
        let _ = weights.set("fps", 30.0);
        let _ = weights.set("h264.qp", 28.0);
        H264Codec::new(16, 16, &weights).expect("codec")
    }

    #[test]
    fn write_read() {
        let mut c = codec();
        let mut cap = SyntheticCapture::new(16, 16);
        let buf = PayloadDump::write(&mut c, &mut cap, 2).unwrap();
        let h = PayloadDump::header(&buf).unwrap();
        let pkts = PayloadDump::packets(&buf).unwrap();
        assert_eq!(h.count as usize, pkts.len());
        assert!(h.bytes > 0);
        assert!(!pkts[0].is_empty());
    }

    #[test]
    fn short_is_none() {
        assert!(PayloadDump::header(&[0u8; 4]).is_none());
        assert!(PayloadDump::packets(&[]).is_none());
    }

    #[test]
    fn differential() {
        let mut ca = codec();
        let mut cb = codec();
        let mut a = SyntheticCapture::new(16, 16);
        let mut b = SyntheticCapture::new(16, 16);
        let ha = PayloadDump::header(&PayloadDump::write(&mut ca, &mut a, 1).unwrap()).unwrap();
        let hb = PayloadDump::header(&PayloadDump::write(&mut cb, &mut b, 3).unwrap()).unwrap();
        assert!(hb.bytes >= ha.bytes);
        assert!(hb.count >= ha.count);
    }
}
