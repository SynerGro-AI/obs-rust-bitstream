//! Safe pure ServiceReport (service + reconnect).
//!
//! One line of the service name and reconnect attempts.
//! No unwraps.
//!
//! Differential: more attempts produce a higher count.

use crate::reconnect::Reconnect;
use crate::service_status::ServiceStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceReport {
    pub name: String,
    pub attempts: u32,
}

impl ServiceReport {
    /// Build from the service status and reconnect.
    pub fn from(service: &ServiceStatus, reconnect: &Reconnect) -> Self {
        Self {
            name: service.name.clone(),
            attempts: reconnect.attempts(),
        }
    }

    /// One line: "name=N attempts=N"
    pub fn one_line(&self) -> String {
        format!("name={} attempts={}", self.name, self.attempts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let s = ServiceStatus {
            name: "twitch".to_string(),
            id: "rtmp_common".to_string(),
        };
        let mut r = Reconnect::new();
        r.start();
        let rep = ServiceReport::from(&s, &r);
        assert_eq!(rep.name, "twitch");
        assert_eq!(rep.attempts, 1);
        assert!(rep.one_line().contains("attempts=1"));
    }

    #[test]
    fn empty_is_zero() {
        let s = ServiceStatus {
            name: String::new(),
            id: String::new(),
        };
        let r = Reconnect::new();
        let rep = ServiceReport::from(&s, &r);
        assert_eq!(rep.name, "");
        assert_eq!(rep.attempts, 0);
    }

    #[test]
    fn differential() {
        let s = ServiceStatus {
            name: "twitch".to_string(),
            id: String::new(),
        };
        let mut a = Reconnect::new();
        let mut b = Reconnect::new();
        a.start();
        b.start();
        b.start();
        let ra = ServiceReport::from(&s, &a);
        let rb = ServiceReport::from(&s, &b);
        assert!(rb.attempts > ra.attempts);
    }
}
