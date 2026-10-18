//! Safe pure RtmpReconnect (RTMP feed + retry).
//!
//! Feeds packets into an RtmpStream. On failure the
//! reconnect counter advances. On success it clears.
//! Empty name is rejected. No unwraps.
//!
//! Differential: a failure produces more attempts.

use crate::encoder_packet::EncoderPacket;
use crate::reconnect::Reconnect;
use crate::rtmp_feed::RtmpFeed;
use crate::rtmp_status::RtmpStatus;
use crate::rtmp_stream::RtmpError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtmpReconnectError {
    Rtmp(RtmpError),
}

pub struct RtmpReconnect;

impl RtmpReconnect {
    /// Feed packets. Success clears the reconnect counter.
    /// Failure advances it. Returns the status on success.
    pub fn feed(
        name: &str,
        key: &str,
        packets: &[EncoderPacket],
        reconnect: &mut Reconnect,
    ) -> Result<RtmpStatus, RtmpReconnectError> {
        match RtmpFeed::feed(name, key, packets) {
            Ok(stream) => {
                reconnect.success();
                Ok(RtmpStatus::from(&stream))
            }
            Err(e) => {
                reconnect.start();
                Err(RtmpReconnectError::Rtmp(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn success_clears() {
        let packets = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1])];
        let mut r = Reconnect::new();
        r.start();
        assert_eq!(r.attempts(), 1);
        let st = RtmpReconnect::feed("live.twitch.tv", "abc", &packets, &mut r).unwrap();
        assert!(st.active);
        assert_eq!(r.attempts(), 0);
    }

    #[test]
    fn failure_advances() {
        let mut r = Reconnect::new();
        assert!(RtmpReconnect::feed("", "abc", &[], &mut r).is_err());
        assert_eq!(r.attempts(), 1);
    }

    #[test]
    fn differential() {
        let packets = vec![EncoderPacket::new(EncoderType::Video, 0, 0, vec![1])];
        let mut a = Reconnect::new();
        let mut b = Reconnect::new();
        let _ = RtmpReconnect::feed("", "k", &[], &mut a);
        let _ = RtmpReconnect::feed("", "k", &[], &mut b);
        let _ = RtmpReconnect::feed("", "k", &[], &mut b);
        let _ = RtmpReconnect::feed("live.twitch.tv", "k", &packets, &mut a);
        assert_eq!(a.attempts(), 0);
        assert!(b.attempts() > a.attempts());
    }
}
