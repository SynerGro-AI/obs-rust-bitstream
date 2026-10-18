//! Safe pure PacketTime (OBS encoder_packet_time).
//!
//! Parsed from libobs/obs-encoder.h.
//! Holds pts, cts, fer, ferc, and pir.
//! Timeline order is CTS, FER, FERC, PIR.
//! No unwraps.
//!
//! Differential: a later pir produces a larger latency.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketTime {
    pub pts: i64,
    pub cts: u64,
    pub fer: u64,
    pub ferc: u64,
    pub pir: u64,
}

impl PacketTime {
    /// Create with a pts. Other times default to zero.
    pub fn new(pts: i64) -> Self {
        Self {
            pts,
            cts: 0,
            fer: 0,
            ferc: 0,
            pir: 0,
        }
    }

    pub fn set_cts(&mut self, cts: u64) {
        self.cts = cts;
    }

    pub fn set_fer(&mut self, fer: u64) {
        self.fer = fer;
    }

    pub fn set_ferc(&mut self, ferc: u64) {
        self.ferc = ferc;
    }

    pub fn set_pir(&mut self, pir: u64) {
        self.pir = pir;
    }

    /// Encode duration: ferc - fer. Zero if ferc is before fer.
    pub fn encode_ns(&self) -> u64 {
        self.ferc.saturating_sub(self.fer)
    }

    /// Total latency: pir - cts. Zero if pir is before cts.
    pub fn latency_ns(&self) -> u64 {
        self.pir.saturating_sub(self.cts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let mut t = PacketTime::new(100);
        t.set_cts(1000);
        t.set_fer(1100);
        t.set_ferc(1200);
        t.set_pir(1500);
        assert_eq!(t.pts, 100);
        assert_eq!(t.encode_ns(), 100);
        assert_eq!(t.latency_ns(), 500);
    }

    #[test]
    fn saturates() {
        let mut t = PacketTime::new(0);
        t.set_fer(100);
        t.set_ferc(50);
        assert_eq!(t.encode_ns(), 0);
    }

    #[test]
    fn differential() {
        let mut a = PacketTime::new(1);
        let mut b = PacketTime::new(2);
        a.set_cts(0);
        a.set_pir(100);
        b.set_cts(0);
        b.set_pir(300);
        assert!(b.latency_ns() > a.latency_ns());
    }
}
