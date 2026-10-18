//! Safe pure LatencyStatus (encode and total latency).
//!
//! Reports encode duration and total latency from a
//! PacketTime. No unwraps.
//!
//! Differential: a later pir produces a larger latency.

use crate::packet_time::PacketTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LatencyStatus {
    pub encode_ns: u64,
    pub latency_ns: u64,
}

impl LatencyStatus {
    /// Build from a PacketTime.
    pub fn from(time: &PacketTime) -> Self {
        Self {
            encode_ns: time.encode_ns(),
            latency_ns: time.latency_ns(),
        }
    }

    /// One line: "encode=N latency=N"
    pub fn one_line(&self) -> String {
        format!("encode={} latency={}", self.encode_ns, self.latency_ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let mut t = PacketTime::new(1);
        t.set_cts(1000);
        t.set_fer(1100);
        t.set_ferc(1300);
        t.set_pir(2000);
        let s = LatencyStatus::from(&t);
        assert_eq!(s.encode_ns, 200);
        assert_eq!(s.latency_ns, 1000);
        assert!(s.one_line().contains("latency=1000"));
    }

    #[test]
    fn zero_is_zero() {
        let t = PacketTime::new(0);
        let s = LatencyStatus::from(&t);
        assert_eq!(s.encode_ns, 0);
        assert_eq!(s.latency_ns, 0);
    }

    #[test]
    fn differential() {
        let mut a = PacketTime::new(1);
        let mut b = PacketTime::new(2);
        a.set_cts(0);
        a.set_pir(100);
        b.set_cts(0);
        b.set_pir(400);
        let sa = LatencyStatus::from(&a);
        let sb = LatencyStatus::from(&b);
        assert!(sb.latency_ns > sa.latency_ns);
    }
}
