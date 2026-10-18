//! Safe pure StreamStatus (streaming + service).
//!
//! One line of streaming state, service name, and bitrate.
//! No unwraps.
//!
//! Differential: a running stream reports streaming=true.

use crate::obs_encoder_defaults::ObsEncoderDefaults;
use crate::program_stream::ProgramStream;
use crate::service_status::ServiceStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamStatus {
    pub streaming: bool,
    pub service: String,
    pub bitrate_kbps: u32,
}

impl StreamStatus {
    /// Build from the stream, service status, and encoder defaults.
    pub fn from(
        stream: &ProgramStream,
        service: &ServiceStatus,
        defaults: &ObsEncoderDefaults,
    ) -> Self {
        Self {
            streaming: stream.is_streaming(),
            service: service.name.clone(),
            bitrate_kbps: defaults.bitrate_kbps,
        }
    }

    /// One line: "streaming=true/false service=N bitrate=N"
    pub fn one_line(&self) -> String {
        format!(
            "streaming={} service={} bitrate={}",
            self.streaming, self.service, self.bitrate_kbps
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let mut s = ProgramStream::new();
        s.start("twitch").unwrap();
        let svc = ServiceStatus {
            name: "twitch".to_string(),
            id: "rtmp_common".to_string(),
        };
        let d = ObsEncoderDefaults::default();
        let st = StreamStatus::from(&s, &svc, &d);
        assert!(st.streaming);
        assert_eq!(st.service, "twitch");
        assert_eq!(st.bitrate_kbps, 6000);
        assert!(st.one_line().contains("streaming=true"));
    }

    #[test]
    fn stopped_is_false() {
        let s = ProgramStream::new();
        let svc = ServiceStatus {
            name: String::new(),
            id: String::new(),
        };
        let d = ObsEncoderDefaults::default();
        let st = StreamStatus::from(&s, &svc, &d);
        assert!(!st.streaming);
    }

    #[test]
    fn differential() {
        let mut a = ProgramStream::new();
        let mut b = ProgramStream::new();
        b.start("youtube").unwrap();
        let svc = ServiceStatus {
            name: String::new(),
            id: String::new(),
        };
        let d = ObsEncoderDefaults::default();
        let sa = StreamStatus::from(&a, &svc, &d);
        let sb = StreamStatus::from(&b, &svc, &d);
        assert!(!sa.streaming);
        assert!(sb.streaming);
    }
}
