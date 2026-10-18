//! Safe pure ServiceReconnect (service change → reset).
//!
//! If the service name changes, the reconnect counter
//! is cleared. The same name leaves it alone. No unwraps.
//!
//! Differential: a different name resets; the same name does not.

use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::reconnect::Reconnect;

pub struct ServiceReconnect;

impl ServiceReconnect {
    /// Set the service. If the name differs from the current,
    /// clear the reconnect counter. Returns true if it changed.
    pub fn set(
        service: &mut OutputService,
        reconnect: &mut Reconnect,
        output: &OutputInfo,
        name: &str,
        id: &str,
    ) -> bool {
        let current = service.service().map(|s| s.name().to_string());
        let changed = current.as_deref() != Some(name);
        if service.set(output, name, id).is_err() {
            return false;
        }
        if changed {
            reconnect.success();
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};

    #[test]
    fn change_resets() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut os = OutputService::new();
        let mut r = Reconnect::new();
        r.start();
        assert_eq!(r.attempts(), 1);
        assert!(ServiceReconnect::set(&mut os, &mut r, &out, "twitch", "rtmp_common"));
        assert_eq!(r.attempts(), 0);
    }

    #[test]
    fn same_keeps() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut os = OutputService::new();
        os.set(&out, "twitch", "rtmp_common").unwrap();
        let mut r = Reconnect::new();
        r.start();
        assert!(!ServiceReconnect::set(&mut os, &mut r, &out, "twitch", "rtmp_common"));
        assert_eq!(r.attempts(), 1);
    }

    #[test]
    fn differential() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut a = OutputService::new();
        let mut b = OutputService::new();
        let mut ra = Reconnect::new();
        let mut rb = Reconnect::new();
        ra.start();
        rb.start();
        ServiceReconnect::set(&mut a, &mut ra, &out, "twitch", "rtmp_common");
        ServiceReconnect::set(&mut b, &mut rb, &out, "twitch", "rtmp_common");
        ServiceReconnect::set(&mut b, &mut rb, &out, "youtube", "rtmp_common");
        assert_eq!(ra.attempts(), 0);
        assert_eq!(rb.attempts(), 0);
    }
}
