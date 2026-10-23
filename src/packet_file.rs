//! Safe pure PacketFile (interleaved packets to bytes).
//!
//! Writes EncoderPackets into a simple byte stream.
//! Header is "PKTF" followed by count and length-prefixed
//! payloads. Empty input returns an empty header. No unwraps.
//!
//! Differential: two packets produce a larger payload than one.

use crate::encoder_packet::EncoderPacket;

pub struct PacketFile;

impl PacketFile {
    /// Write packets to a byte buffer.
    /// Format: "PKTF" + u32 count + for each: u32 len + data.
    pub fn write(packets: &[EncoderPacket]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"PKTF");
        let count = packets.len() as u32;
        out.extend_from_slice(&count.to_le_bytes());
        for pkt in packets {
            let len = pkt.data.len() as u32;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&pkt.data);
        }
        out
    }

    /// Read the packet count from a buffer written by write.
    /// Returns None if the header is missing or truncated.
    pub fn count(buf: &[u8]) -> Option<usize> {
        if buf.len() < 8 || &buf[0..4] != b"PKTF" {
            return None;
        }
        let count = u32::from_le_bytes(buf[4..8].try_into().ok()?);
        Some(count as usize)
    }

    /// Read payloads from a buffer written by write.
    /// Returns None if the header is missing or a length is truncated.
    pub fn read(buf: &[u8]) -> Option<Vec<Vec<u8>>> {
        if buf.len() < 8 || &buf[0..4] != b"PKTF" {
            return None;
        }
        let count = u32::from_le_bytes(buf[4..8].try_into().ok()?) as usize;
        let mut out = Vec::with_capacity(count);
        let mut pos = 8;
        for _ in 0..count {
            if pos + 4 > buf.len() {
                return None;
            }
            let len = u32::from_le_bytes(buf[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            if pos + len > buf.len() {
                return None;
            }
            out.push(buf[pos..pos + len].to_vec());
            pos += len;
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;
    use crate::encoder_packet::EncoderPacket;

    #[test]
    fn write_one() {
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2, 3]);
        let buf = PacketFile::write(&[pkt]);
        assert!(buf.starts_with(b"PKTF"));
        assert_eq!(PacketFile::count(&buf), Some(1));
    }

    #[test]
    fn empty_is_header() {
        let buf = PacketFile::write(&[]);
        assert_eq!(PacketFile::count(&buf), Some(0));
    }

    #[test]
    fn bad_header() {
        assert!(PacketFile::count(b"BAD").is_none());
        assert!(PacketFile::count(&[]).is_none());
    }

    #[test]
    fn round_trip() {
        let a = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]);
        let b = EncoderPacket::new(EncoderType::Audio, 1, 1, vec![3]);
        let buf = PacketFile::write(&[a, b]);
        let payloads = PacketFile::read(&buf).unwrap();
        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0], vec![1, 2]);
        assert_eq!(payloads[1], vec![3]);
    }

    #[test]
    fn differential() {
        let a = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        let b = EncoderPacket::new(EncoderType::Audio, 0, 0, vec![2, 3]);
        let one = PacketFile::write(&[a]);
        let two = PacketFile::write(&[EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]), b]);
        assert!(two.len() > one.len());
        assert_eq!(PacketFile::count(&two), Some(2));
    }
}
