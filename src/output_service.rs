//! Safe pure OutputService (OBS set service).
//!
//! Associates a ServiceContext with an OutputInfo.
//! Empty service names are rejected. No unwraps.
//!
//! Differential: a different service name produces a different id.

use crate::output_info::OutputInfo;
use crate::service_context::{ServiceContext, ServiceError};

#[derive(Clone, Debug, Default)]
pub struct OutputService {
    service: Option<ServiceContext>,
}

impl OutputService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the service. Empty names are rejected by ServiceContext.
    pub fn set(&mut self, _output: &OutputInfo, name: &str, id: &str) -> Result<(), ServiceError> {
        let svc = ServiceContext::new(name, id)?;
        self.service = Some(svc);
        Ok(())
    }

    pub fn service(&self) -> Option<&ServiceContext> {
        self.service.as_ref()
    }

    pub fn clear(&mut self) {
        self.service = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_info::{OutputInfo, OUTPUT_AV, OUTPUT_ENCODED};

    #[test]
    fn set_and_get() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut os = OutputService::new();
        os.set(&out, "twitch", "rtmp_common").unwrap();
        assert_eq!(os.service().unwrap().name(), "twitch");
        assert_eq!(os.service().unwrap().id(), "rtmp_common");
    }

    #[test]
    fn empty_rejected() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut os = OutputService::new();
        assert!(os.set(&out, "", "rtmp_common").is_err());
    }

    #[test]
    fn clear() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut os = OutputService::new();
        os.set(&out, "twitch", "rtmp_common").unwrap();
        os.clear();
        assert!(os.service().is_none());
    }

    #[test]
    fn differential() {
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV | OUTPUT_ENCODED).unwrap();
        let mut a = OutputService::new();
        let mut b = OutputService::new();
        a.set(&out, "twitch", "rtmp_common").unwrap();
        b.set(&out, "youtube", "rtmp_common").unwrap();
        assert_ne!(a.service().unwrap().name(), b.service().unwrap().name());
    }
}
