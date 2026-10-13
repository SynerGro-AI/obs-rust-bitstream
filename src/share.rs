//! Safe pure Share (export to target core).
//!
//! Posts an export payload to a target. Empty targets
//! rejected. No unwraps.
//!
//! Differential: share + target round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShareError {
    EmptyTarget,
    EmptyPayload,
}

#[derive(Clone, Debug)]
pub struct Share {
    target: String,
    bytes: usize,
}

impl Share {
    /// Post a payload to a target. Returns the share record.
    pub fn post(target: &str, payload: &[u8]) -> Result<Self, ShareError> {
        if target.is_empty() {
            return Err(ShareError::EmptyTarget);
        }
        if payload.is_empty() {
            return Err(ShareError::EmptyPayload);
        }
        Ok(Self {
            target: target.to_string(),
            bytes: payload.len(),
        })
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_target() {
        assert!(Share::post("", b"GIF|x|1").is_err());
    }

    #[test]
    fn differential() {
        let s = Share::post("x.com/live", b"GIF|loop.gif|2").unwrap();
        assert_eq!(s.target(), "x.com/live");
        assert_eq!(s.bytes(), 14);
    }
}
