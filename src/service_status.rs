//! Safe pure ServiceStatus (service name report).
//!
//! One line of the service name and id.
//! Empty if no service is set. No unwraps.
//!
//! Differential: a different name produces a different line.

use crate::output_service::OutputService;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceStatus {
    pub name: String,
    pub id: String,
}

impl ServiceStatus {
    /// Build from an OutputService.
    pub fn from(service: &OutputService) -> Self {
        match service.service() {
            Some(s) => Self {
                name: s.name().to_string(),
                id: s.id().to_string(),
            },
            None => Self {
                name: String::new(),
                id: String::new(),
            },
        }
    }

    /// True if a service is set.
    pub fn is_set(&self) -> bool {
        !self.name.is_empty()
    }

    /// One line: "name=N id=N" or "name= id=".
    pub fn one_line(&self) -> String {
        format!("name={} id={}", self.name, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};

    #[test]
    fn reports() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut os = OutputService::new();
        os.set(&out, "twitch", "rtmp_common").unwrap();
        let s = ServiceStatus::from(&os);
        assert!(s.is_set());
        assert_eq!(s.name, "twitch");
        assert!(s.one_line().contains("name=twitch"));
    }

    #[test]
    fn empty_is_empty() {
        let os = OutputService::new();
        let s = ServiceStatus::from(&os);
        assert!(!s.is_set());
        assert_eq!(s.name, "");
    }

    #[test]
    fn differential() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut a = OutputService::new();
        let mut b = OutputService::new();
        a.set(&out, "twitch", "rtmp_common").unwrap();
        b.set(&out, "youtube", "rtmp_common").unwrap();
        let sa = ServiceStatus::from(&a);
        let sb = ServiceStatus::from(&b);
        assert_ne!(sa.name, sb.name);
    }
}
