//! Safe pure RtmpOutput (file packets → RTMP stream).
//!
//! Reads a PayloadDump buffer and feeds every packet into
//! an RtmpStream. The file dump stays the always-on path.
//! Empty URL is rejected. No unwraps.
//!
//! Differential: more packets produce more bytes sent.

use crate::encoder_context::EncoderType;
use crate::encoder_packet::EncoderPacket;
use crate::payload_dump::PayloadDump;
use crate::rtmp_feed::RtmpFeed;
use crate::rtmp_status::RtmpStatus;
use crate::rtmp_stream::RtmpError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtmpOutError {
    Rtmp(RtmpError),
    BadPayload,
}

pub struct RtmpOutput;

impl RtmpOutput {
    /// Feed the packets from a PayloadDump buffer into an RTMP stream.
    /// Returns the status. File dump is unchanged.
    pub fn send(name: &str, key: &str, payload: &[u8]) -> Result<RtmpStatus, RtmpOutError> {
        let packets = PayloadDump::packets(payload).ok_or(RtmpOutError::BadPayload)?;
        let encoded: Vec<EncoderPacket> = packets
            .into_iter()
            .enumerate()
            .map(|(i, data)| EncoderPacket::new(EncoderType::Video, i as i64, i as i64, data))
            .collect();
        let stream = RtmpFeed::feed(name, key, &encoded).map_err(RtmpOutError::Rtmp)?;
        Ok(RtmpStatus::from(&stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::SyntheticCapture;
    use crate::h264_codec::H264Codec;
    use crate::live_weights::LiveWeights;

    fn payload(frames: usize) -> Vec<u8> {
        let mut w = LiveWeights::new();
        let _ = w.set("fps", 30.0);
        let _ = w.set("h264.qp", 28.0);
        let mut codec = H264Codec::new(16, 16, &w).unwrap();
        let mut cap = SyntheticCapture::new(16, 16);
        PayloadDump::write(&mut codec, &mut cap, frames).unwrap()
    }

    #[test]
    fn sends() {
        let buf = payload(2);
        let st = RtmpOutput::send("live.twitch.tv", "abc", &buf).unwrap();
        assert!(st.active);
        assert!(st.packets >= 1);
        assert!(st.bytes > 0);
        assert!(st.one_line().contains("active=true"));
    }

    #[test]
    fn empty_rejected() {
        let buf = payload(1);
        assert!(RtmpOutput::send("", "abc", &buf).is_err());
    }

    #[test]
    fn bad_payload() {
        assert!(RtmpOutput::send("live.twitch.tv", "abc", b"bad").is_err());
    }

    #[test]
    fn differential() {
        let a = RtmpOutput::send("a", "k", &payload(1)).unwrap();
        let b = RtmpOutput::send("b", "k", &payload(3)).unwrap();
        assert!(b.bytes >= a.bytes);
    }
}
