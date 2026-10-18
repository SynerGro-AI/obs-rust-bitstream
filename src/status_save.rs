//! Safe pure StatusSave (packets + status together).
//!
//! Writes due packets and the full status line.
//! Returns both buffers. No unwraps.
//!
//! Differential: a due packet produces a larger packet buffer.

use crate::delay_file::DelayFile;
use crate::encoder_packet::EncoderPacket;
use crate::full_status::FullStatus;
use crate::output_delay_queue::OutputDelayQueue;
use crate::status_file::StatusFile;

pub struct StatusSave;

impl StatusSave {
    /// Push a packet and write due payloads plus the status.
    /// Returns (packet_buf, status_buf).
    pub fn push(
        queue: &mut OutputDelayQueue,
        status: &FullStatus,
        packet: EncoderPacket,
        now: u64,
    ) -> (Vec<u8>, Vec<u8>) {
        let packets = DelayFile::push_and_write(queue, packet, now);
        let status_buf = StatusFile::write(status);
        (packets, status_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caption_status::CaptionStatus;
    use crate::encoder_context::EncoderType;
    use crate::keyframe_status::KeyframeStatus;
    use crate::output_pause::OutputPause;
use crate::av_status::AvStatus;
use crate::service_status::ServiceStatus;
    use crate::path_status::PathStatus;
    use crate::reconnect::Reconnect;
    use crate::roi_status::RoiStatus;

    fn status() -> FullStatus {
        FullStatus::from(
            &PathStatus {
                held: 0,
                dropped: 0,
                latency_ns: 0,
            },
            &KeyframeStatus {
                required: 0,
                done: 0,
                complete: true,
            },
            &CaptionStatus {
                lines: 0,
                duration: 0.0,
            },
            &RoiStatus {
                count: 0,
                supported: false,
            },
            &Reconnect::new(),
            &OutputPause::new(),
            &AvStatus { pixels: 0, rate: 0, channels: 0, matches: true },
            &ServiceStatus { name: String::new(), id: String::new() },
        )
    }

    #[test]
    fn writes_both() {
        let mut q = OutputDelayQueue::new(0);
        let s = status();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        let (p, st) = StatusSave::push(&mut q, &s, pkt, 0);
        assert!(StatusFile::is_status(&st));
        assert_eq!(crate::packet_file::PacketFile::count(&p), Some(0));
    }

    #[test]
    fn due_writes_packets() {
        let mut q = OutputDelayQueue::new(50);
        let s = status();
        StatusSave::push(
            &mut q,
            &s,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        let (p, _) = StatusSave::push(
            &mut q,
            &s,
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]),
            51,
        );
        assert!(crate::packet_file::PacketFile::count(&p).unwrap() >= 1);
    }

    #[test]
    fn differential() {
        let mut a = OutputDelayQueue::new(0);
        let mut b = OutputDelayQueue::new(0);
        let s = status();
        let (pa, _) = StatusSave::push(
            &mut a,
            &s,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        StatusSave::push(
            &mut b,
            &s,
            EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]),
            0,
        );
        let (pb, _) = StatusSave::push(
            &mut b,
            &s,
            EncoderPacket::new(EncoderType::Video, 1, 1, vec![2]),
            1,
        );
        assert!(pb.len() >= pa.len());
    }
}
