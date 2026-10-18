//! Safe pure RtmpStream (OBS rtmp_stream shape).
//!
//! Parsed from plugins/obs-outputs/rtmp-stream.h.
//! Tracks active, disconnected, sent_headers, and got_first_packet.
//! Accepts packets and counts bytes. Empty URL is rejected.
//! No unwraps.
//!
//! Differential: more packets produce more bytes.

use crate::encoder_packet::EncoderPacket;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtmpError {
    EmptyUrl,
    Inactive,
}

#[derive(Clone, Debug)]
pub struct RtmpStream {
    url: String,
    active: bool,
    disconnected: bool,
    sent_headers: bool,
    got_first_packet: bool,
    bytes: u64,
    packets: usize,
}

impl RtmpStream {
    /// Create with a URL. Empty URL is rejected.
    pub fn new(url: &str) -> Result<Self, RtmpError> {
        if url.is_empty() {
            return Err(RtmpError::EmptyUrl);
        }
        Ok(Self {
            url: url.to_string(),
            active: false,
            disconnected: false,
            sent_headers: false,
            got_first_packet: false,
            bytes: 0,
            packets: 0,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }

    pub fn sent_headers(&self) -> bool {
        self.sent_headers
    }

    pub fn got_first_packet(&self) -> bool {
        self.got_first_packet
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn packet_count(&self) -> usize {
        self.packets
    }

    /// Mark connecting then active. Matches OBS connect → active.
    pub fn connect(&mut self) {
        self.active = true;
        self.disconnected = false;
    }

    /// Accept a packet. Inactive streams reject. First packet sets the flag.
    pub fn push(&mut self, packet: &EncoderPacket) -> Result<(), RtmpError> {
        if !self.active {
            return Err(RtmpError::Inactive);
        }
        if !self.sent_headers {
            self.sent_headers = true;
        }
        if !self.got_first_packet {
            self.got_first_packet = true;
        }
        self.bytes = self.bytes.saturating_add(packet.data.len() as u64);
        self.packets = self.packets.saturating_add(1);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.active = false;
        self.disconnected = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn connect_and_push() {
        let mut s = RtmpStream::new("rtmp://live.twitch.tv/app").unwrap();
        assert!(!s.is_active());
        s.connect();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2, 3]);
        s.push(&pkt).unwrap();
        assert!(s.is_active());
        assert!(s.sent_headers());
        assert!(s.got_first_packet());
        assert_eq!(s.bytes(), 3);
        assert_eq!(s.packet_count(), 1);
    }

    #[test]
    fn rejects() {
        assert!(RtmpStream::new("").is_err());
        let mut s = RtmpStream::new("rtmp://x").unwrap();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        assert!(s.push(&pkt).is_err());
    }

    #[test]
    fn disconnect() {
        let mut s = RtmpStream::new("rtmp://x").unwrap();
        s.connect();
        s.disconnect();
        assert!(!s.is_active());
        assert!(s.is_disconnected());
    }

    #[test]
    fn differential() {
        let mut a = RtmpStream::new("rtmp://a").unwrap();
        let mut b = RtmpStream::new("rtmp://b").unwrap();
        a.connect();
        b.connect();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1, 2]);
        a.push(&pkt).unwrap();
        b.push(&pkt).unwrap();
        b.push(&pkt).unwrap();
        assert!(b.bytes() > a.bytes());
    }
}
